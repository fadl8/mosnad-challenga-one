use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::diesel::prelude::*;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Selectable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub is_admin: bool,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
}

#[derive(Insertable, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub password: String,
}

diesel::table! {
    users (id) {
        id -> Int4,
        is_admin -> Bool,
        email -> Text,
        password -> Text,
     }
}
