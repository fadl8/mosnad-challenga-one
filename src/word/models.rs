use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::diesel::prelude::*;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Selectable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = words)]
pub struct Word {
    pub id: i32, 
    pub title: String,
    pub description: String,
    pub character: String,
    pub approved: bool,
    pub userId: i32, 
}

#[derive(Insertable, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = words)]
pub struct NewWord {
    pub title: String,
    pub description: String,
    pub character: String,
    pub approved: bool,
    pub userId: i32
}

diesel::table! {
    words (id) {
        id -> Int4,
        title -> Text,
        description -> Text,
        character -> Text,
        approved -> Bool,
        userId -> Int4
     }
}
