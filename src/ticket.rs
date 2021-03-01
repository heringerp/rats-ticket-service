extern crate rocket_multipart_form_data;

/* Diesel query builder */
use diesel::prelude::*;

/* Database macros */
use crate::schema::*;

/* Database data structs */
use crate::models::*;

/* Flash message and redirect */
use rocket::http::ContentType;
use rocket::response::{Flash, Redirect};
use rocket::Data;
use rocket_contrib::templates::Template;

use hyperx::header::{DispositionParam, DispositionType};
use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{header, Message, SmtpTransport, Transport};

use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};

use chrono::Local;
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn send_email(
    to: &str,
    subject: &str,
    body: &str,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Filename: {}", filename);
    let email_address = to.parse()?;
    let file = std::fs::read(filename)?;
    let email = (Message::builder()
        .from("Request-Approval-Ticket-Service <test@test.com>".parse()?)
        .to(email_address)
        .subject(subject)
        .multipart(
            MultiPart::mixed()
                .singlepart(
                    SinglePart::quoted_printable()
                        .header(header::ContentType(
                            "text/plain; charset=utf8".parse().unwrap(),
                        ))
                        .body(body),
                )
                .singlepart(
                    SinglePart::base64()
                        .header(header::ContentType("application/msword".parse().unwrap()))
                        .header(header::ContentDisposition {
                            disposition: DispositionType::Attachment,
                            parameters: vec![DispositionParam::Filename(
                                header::Charset::Ext("utf-8".into()),
                                None,
                                filename.as_bytes().into(),
                            )],
                        })
                        .body(file),
                ),
        ))?;

    let gmail_address = env::var("GMAIL_USERNAME")?;
    let gmail_password = env::var("GMAIL_PASSWORD")?;

    let creds = Credentials::new(gmail_address, gmail_password);

    // Open a remote connection to gmail
    let relay = SmtpTransport::relay("smtp.gmail.com")?;
    let mailer = relay.credentials(creds).build();

    // Send the email
    mailer.send(&email)?;
    Ok(())
}

pub fn all_to_request(
    conn: &SqliteConnection,
    user_id: usize,
) -> Result<Vec<Ticket>, diesel::result::Error> {
    tickets::table
        .filter(tickets::requestor.eq(user_id as i32))
        .filter(tickets::approved.eq(false))
        .select(tickets::all_columns)
        .load::<Ticket>(conn)
}

pub fn all_to_approve(
    conn: &SqliteConnection,
    user_id: usize,
) -> Result<Vec<Ticket>, diesel::result::Error> {
    tickets::table
        .filter(tickets::approver.eq(user_id as i32))
        .filter(tickets::approved.eq(false))
        .select(tickets::all_columns)
        .load::<Ticket>(conn)
}

pub fn insert(
    ticket: &TicketValues,
    conn: &SqliteConnection,
    user_id: usize,
) -> Result<(), diesel::result::Error> {
    use crate::schema::accounts::dsl::*;
    let approver_id: Option<i32> = accounts
        .filter(email.eq(String::from(&ticket.approver_email)))
        .select(id)
        .first(conn)?;

    let approver_id = match approver_id {
        Some(x) => x,
        None => return Err(diesel::result::Error::NotFound),
    };
    let requestor_id = user_id as i32;

    // if approver_id == requestor_id {
    //     println!("###---### Approver equals requestor ###---###");
    //     return false;
    // }

    let t = Ticket {
        id: None,
        description: String::from(&ticket.description),
        approved: false,
        approver: approver_id,
        requestor: requestor_id,
        filename: String::from(&ticket.filename),
    };
    //println!("{}\n", ticket.upload_file);
    match diesel::insert_into(tickets::table).values(&t).execute(conn) {
        Ok(_) => Ok(()),
        Err(x) => Err(x),
    }
}

pub fn toggle_with_id(
    ide: i32,
    conn: &SqliteConnection,
) -> Result<(), (bool, Box<dyn std::error::Error>)> {
    let ticket = tickets::table.find(ide).get_result::<Ticket>(conn);
    if ticket.is_err() {
        return Err((false, Box::new(diesel::result::Error::NotFound)));
    }

    let updated_ticket = diesel::update(tickets::table.find(ide));
    match updated_ticket.set(tickets::approved.eq(true)).execute(conn) {
        Ok(_) => (),
        Err(x) => return Err((false, Box::new(x))),
    };

    let (descript, approver_id, requestor_id, filename): (String, i32, i32, String) =
        match tickets::table
            .filter(tickets::id.eq(ide))
            .select((
                tickets::description,
                tickets::approver,
                tickets::requestor,
                tickets::filename,
            ))
            .first(conn)
        {
            Ok(o) => o,
            Err(e) => return Err((true, Box::new(e))),
        };

    let (re_email, re_firstname, re_lastname): (String, Option<String>, String) =
        match accounts::table
            .filter(accounts::id.eq(requestor_id))
            .select((accounts::email, accounts::firstname, accounts::lastname))
            .first(conn)
        {
            Ok(o) => o,
            Err(e) => return Err((true, Box::new(e))),
        };

    let (ap_email, ap_firstname, ap_lastname): (String, Option<String>, String) =
        match accounts::table
            .filter(accounts::id.eq(approver_id))
            .select((accounts::email, accounts::firstname, accounts::lastname))
            .first(conn)
        {
            Ok(o) => o,
            Err(e) => return Err((true, Box::new(e))),
        };

    let text = format!("The following request was approved:\n\n{}\n\nRequested by: {}, {} ({})\nApproved by: {}, {} ({})",
    descript,
    re_lastname, re_firstname.unwrap_or("".to_string()), re_email,
    ap_lastname, ap_firstname.unwrap_or("".to_string()), ap_email);

    let subject = format!("Approval of request #{}", ide);

    let admin_email = match env::var("ADMIN_EMAIL") {
        Ok(o) => o,
        Err(e) => return Err((true, Box::new(e))),
    };

    match send_email(
        &admin_email[..],
        &subject,
        &text,
        &format!("uploads/{}", filename),
    ) {
        Ok(_) => (),
        Err(e) => return Err((true, e)),
    }

    Ok(())
}

pub fn delete_with_id(ide: i32, conn: &SqliteConnection) -> Result<usize, diesel::result::Error> {
    use crate::schema::tickets::dsl::*;
    diesel::delete(tickets.find(ide)).execute(conn)
}

// #[cfg(test)]
// pub fn delete_all(conn: &SqliteConnection) -> bool {
//     diesel::delete(all_tickets).execute(conn).is_ok()
// }

#[get("/")]
pub fn new(_user: crate::User) -> Template {
    use std::collections::HashMap;
    let m: HashMap<i32, i32> = HashMap::new();
    Template::render("create_ticket", m)
}

fn write_file(bytes: Vec<u8>) -> std::io::Result<String> {
    let filename = format!(
        "{}-{}.doc",
        Local::now().format("%Y%m%dT%H%M%S"),
        rand::random::<u32>()
    );
    let mut file = File::create(format!("uploads/{}", &filename))?;
    file.write_all(&bytes)?;
    return Ok(filename);
}

#[post("/", data = "<paste>")]
pub fn add_new(content_type: &ContentType, paste: Data, user: crate::User) -> Flash<Redirect> {
    println!("A");
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::text("description"),
        MultipartFormDataField::text("approver_email"),
        MultipartFormDataField::raw("upload_file").size_limit(100 * 1024),
    ]);
    println!("B");

    let mut multipart_form_data = match MultipartFormData::parse(content_type, paste, options) {
        Ok(x) => x,
        Err(_) => return Flash::error(Redirect::to("/"), "Ungültige Datei"),
    };

    println!("C");
    let upload_file = multipart_form_data.raw.remove("upload_file");
    let description = multipart_form_data.texts.remove("description");
    let approver_email = multipart_form_data.texts.remove("approver_email");

    println!("Getting values");
    let mut description_fields = match description {
        Some(x) => x,
        None => {
            return Flash::error(
                Redirect::to("/"),
                "Ticket-Beschreibung konnte nicht geladen werden",
            )
        }
    };
    println!("Got description");
    let mut approver_fields = match approver_email {
        Some(x) => x,
        None => {
            return Flash::error(
                Redirect::to("/"),
                "Genehmiger-Email konnte nicht geladen werden",
            )
        }
    };
    println!("Got email");
    let mut raw_fields = match upload_file {
        Some(x) => x,
        None => return Flash::error(Redirect::to("/"), "Datei konnte nicht geladen werden"),
    };
    println!("Got file");
    let raw_field = raw_fields.remove(0);
    let description_field = description_fields.remove(0);
    let approver_field = approver_fields.remove(0);
    let raw_filename = match raw_field.file_name {
        Some(x) => x,
        None => return Flash::error(Redirect::to("/"), "Datei konnte nicht geladen werden"),
    };
    let raw_file = raw_field.raw;
    let description = description_field.text;
    let approver_email = approver_field.text;

    let filename = match write_file(raw_file) {
        Ok(x) => x,
        Err(_) => return Flash::error(Redirect::to("/"), "Datei konnte nicht hochgeladen werden"),
    };
    // You can now deal with the text data.
    println!("Description: {}", description);
    println!("Approver Email: {}", approver_email);
    println!("Upload filename: {}", raw_filename);
    println!("Filename on server: {}", filename);

    let ticketvalues = TicketValues {
        description,
        approver_email,
        filename,
    };

    match insert(&ticketvalues, &crate::establish_connection(), user.0) {
        Ok(_) => (),
        Err(_) => return Flash::error(Redirect::to("/"), "Ticket konnte nicht erstellt werden"),
    };

    let rats_url = match env::var("RATS_URL") {
        Ok(x) => x,
        Err(_) => String::from(""),
    };
    match send_email(
        &ticketvalues.approver_email,
        "New request to approve",
        &format!(
            "
You have a new request about:

{}

Please approve or deny it here: {}",
            &ticketvalues.description, &rats_url
        ),
        &format!("uploads/{}", &ticketvalues.filename),
    ) {
        Ok(_) => (),
        Err(_) => return Flash::error(Redirect::to("/"), "Bestätigungsemail konnte nicht versendet werden"),
    };

    Flash::success(Redirect::to("/"), "Ticket wurde erfolgreich hinzugefügt")
}

#[put("/<id>")]
pub fn toggle(id: i32) -> Result<Redirect, Flash<Redirect>> {
    match toggle_with_id(id, &crate::establish_connection()) {
        Ok(_) => Ok(Redirect::to("/")),
        Err((x, y)) if !x => {
            eprintln!("{}", y.to_string());
            Err(Flash::error(
                Redirect::to("/"),
                "Ticket konnte nicht genehmigt werden",
            ))
        }
        Err((x, y)) if x => {
            eprintln!("{}", y.to_string());
            Err(Flash::error(Redirect::to("/"), "Ticket wurde genehmigt, Bestätigungsemail konnte allerdings nicht versendet werden"))
        }
        Err(_) => {
            eprintln!("Error, ran into serious trouble when toggling ids");
            Err(Flash::error(Redirect::to("/"), "Etwas ist schief gelaufen"))
        }
    }
}

#[delete("/<id>")]
pub fn delete(id: i32) -> Result<Flash<Redirect>, Flash<Redirect>> {
    match delete_with_id(id, &crate::establish_connection()) {
        Ok(_) => Ok(Flash::success(
            Redirect::to("/"),
            "Ticket wurde erfolgreich gelöscht",
        )),
        Err(x) => {
            eprintln!("{}", x.to_string());
            Err(Flash::error(
                Redirect::to("/"),
                "Ticket konnte nicht gelöscht werden",
            ))
        }
    }
}

pub struct TicketValues {
    pub description: String,
    pub approver_email: String,
    pub filename: String,
}
