#![feature(proc_macro_hygiene, decl_macro, never_type)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate log;
#[macro_use]
extern crate serde_derive;

//#[cfg(test)] mod tests;

use std::env;

use diesel::SqliteConnection;
use request::FlashMessage;use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::Redirect;
use rocket_contrib::{serve::StaticFiles, templates::Template};

embed_migrations!();

pub mod models;
pub mod schema;
pub mod ticket;
pub mod user;

use models::*;
use schema::*;

extern crate dotenv;

use std::fs::File;

use diesel::prelude::*;
use dotenv::dotenv;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[derive(FromForm)]
pub struct Login {
    username: String,
    password: String,
}

#[derive(Debug)]
pub struct User(usize);

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, !> {
        request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| User(id))
            .or_forward(())
    }
}

#[derive(Serialize)]
struct TemplateContext {
    user_id: usize,
    request_ticket: Vec<models::Ticket>,
    approval_ticket: Vec<models::Ticket>,
    flash: Option<String>
}

#[get("/")]
fn user_index(user: User, flash: Option<FlashMessage>) -> Template {
    let conn = establish_connection();
    let all_to_request = match ticket::all_to_request(&conn, user.0) {
        Ok(o) => o,
        Err(_) => Vec::new(),
    };
    let all_to_approve = match ticket::all_to_approve(&conn, user.0) {
        Ok(o) => o,
        Err(_) => Vec::new(),
    };
    let flash_string = match flash {
        Some(x) => Some(x.msg().to_string()),
        None => None,
    };
    let context = TemplateContext {
        user_id: user.0,
        request_ticket: all_to_request,
        approval_ticket: all_to_approve,
        flash: flash_string,
    };
    Template::render("index", &context)
}

#[get("/template.doc")]
fn retrieve_template(_user: User) -> Option<File> {
    let filename = "uploads/template.doc";
    File::open(&filename).ok()
}

#[get("/<id>", rank = 3)]
fn retrieve_file(id: String, _user: User) -> Option<File> {
    let conn = establish_connection();
    let entries = tickets::table
        .filter(tickets::filename.eq(&id))
        .filter(
            tickets::approver
                .eq(_user.0 as i32)
                .or(tickets::requestor.eq(_user.0 as i32)),
        )
        .select(tickets::all_columns)
        .load::<Ticket>(&conn)
        .expect("Whoops! Something went bananas.");
    // If nothing is found user isn't allowed to view file
    if entries.len() < 1 {
        return None;
    }
    let filename = format!("uploads/{}", id);
    File::open(&filename).ok()
}

#[get("/", rank = 2)]
fn index() -> Redirect {
    Redirect::to(uri!(user::login_page))
}

// #[post("/upload", format = "plain", data = "<data>")]
// async fn upload(data: Data) -> Result<String, Debug<io::Error>> {
//     let path = env::temp_dir().join("upload.txt");
//     Ok(data.open(128.kibibytes()).stream_to_file(path).await?.to_string())
// }

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount(
            "/",
            routes![
                index,
                user_index,
                retrieve_template,
                retrieve_file,
                user::login,
                user::logout,
                user::login_user,
                user::login_page,
                user::signup_user,
                user::new_account,
                user::signup_page
            ],
        )
        .mount(
            "/",
            StaticFiles::from("static")
        )
        .mount("/ticket", routes![ticket::toggle, ticket::delete])
        .mount("/new", routes![ticket::new, ticket::add_new])
        .attach(Template::fairing())
}

fn main() {
    println!("--------------------------");
    println!("--------------------------");
    let conn = establish_connection();
    match embedded_migrations::run(&conn) {
        Err(_) => eprintln!("Diesel migrations could not be run"),
        _ => (),
    }

    rocket().launch();
}
