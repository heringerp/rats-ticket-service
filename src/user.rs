use diesel::prelude::*;

use crate::models::*;
use crate::schema::*;

/* To be able to return Templates */
use rocket_contrib::templates::Template;
use std::collections::HashMap;

/* To be able to parse raw forms */
use rocket::http::{Cookie, Cookies};

/* Flash message and redirect */
use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;

fn insert(user: AccountForm, conn: &SqliteConnection) -> bool {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password_simple(user.password.as_bytes(), salt.as_ref())
        .unwrap()
        .to_string();

    let acc = Account {
        id: None,
        email: user.email,
        firstname: user.firstname,
        lastname: user.lastname,
        password: password_hash,
    };
    diesel::insert_into(accounts::table)
        .values(&acc)
        .execute(conn)
        .is_ok()
}

#[post("/signup", data = "<signup_form>")]
pub fn new_account(signup_form: Form<AccountForm>) -> Flash<Redirect> {
    let user = signup_form.into_inner();
    if user.email.is_empty() {
        Flash::error(Redirect::to("/signup"), "Email darf nicht leer sein")
    } else if user.password.is_empty() || user.repeat_password.is_empty() {
        Flash::error(Redirect::to("/signup"), "Password darf nicht leer sein")
    } else if user.password != user.repeat_password {
        // compare passwords
        Flash::error(Redirect::to("/signup"), "Passwörter stimmen nicht überein")
    } else if insert(user, &crate::establish_connection()) {
        Flash::success(Redirect::to("/"), "Account wurde erfolgreich erstellt")
    } else {
        Flash::error(Redirect::to("/signup"), "Probleme bei der Accounterstellung")
    }
}

fn get_id(
    input_email: &str,
    input_password: &str,
    conn: &SqliteConnection,
) -> Result<Option<i32>, diesel::result::Error> {
    use crate::schema::accounts::dsl::*;
    println!("Email: {}", input_email);
    let password_string = accounts
        .filter(email.eq(input_email))
        .select(password)
        .first::<String>(conn)?;
    let parsed_hash = PasswordHash::new(&password_string).unwrap();
    let argon2 = Argon2::default();
    if !argon2
        .verify_password(input_password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        return Err(diesel::NotFound);
    }
    accounts
        .filter(email.eq(input_email))
        .select(id)
        .first(conn)
}

#[post("/login", data = "<login>")]
pub fn login(mut cookies: Cookies, login: Form<crate::Login>) -> Result<Redirect, Flash<Redirect>> {
    match get_id(
        &login.username,
        &login.password,
        &crate::establish_connection(),
    ) {
        Ok(id) => {
            cookies.add_private(Cookie::new("user_id", id.unwrap().to_string()));
            Ok(Redirect::to(uri!(crate::index)))
        }
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(login_page)),
            "Ungültige Email oder Password",
        )),
    }
}

#[post("/logout")]
pub fn logout(mut cookies: Cookies) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("user_id"));
    Flash::success(Redirect::to(uri!(login_page)), "Erfolgreich abgemeldet")
}

#[get("/login")]
pub fn login_user(_user: crate::User) -> Redirect {
    Redirect::to("/")
}

#[get("/login", rank = 2)]
pub fn login_page(flash: Option<FlashMessage>) -> Template {
    let mut context = HashMap::new();
    if let Some(ref msg) = flash {
        context.insert("flash", msg.msg());
    }

    Template::render("login", &context)
}

#[get("/signup")]
pub fn signup_user(_user: crate::User) -> Redirect {
    Redirect::to("/")
}

#[get("/signup", rank = 2)]
pub fn signup_page(flash: Option<FlashMessage>) -> Template {
    let mut context = HashMap::new();
    if let Some(ref msg) = flash {
        context.insert("flash", msg.msg());
    }

    Template::render("signup", &context)
}

#[derive(FromForm)]
pub struct AccountForm {
    pub email: String,
    pub firstname: String,
    pub lastname: String,
    pub password: String,
    pub repeat_password: String,
}
