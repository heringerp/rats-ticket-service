/* Import macros and others */
use crate::schema::*;

#[derive(Serialize, Queryable, Insertable, Debug, Clone)]
#[table_name = "tickets"]
pub struct Ticket {
    pub id: Option<i32>,
    pub description: String,
    pub approved: bool,
    pub approver: i32,
    pub requestor: i32,
    pub filename: String,
}

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "accounts"]
pub struct Account {
    pub id: Option<i32>,
    pub email: String,
    pub firstname: String,
    pub lastname: String,
    pub password: String,
}
