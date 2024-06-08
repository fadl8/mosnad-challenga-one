use database::Db;
use rocket_db_pools::{Connection, Database};


#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_db_pools;

use dotenvy::dotenv;

mod jwt;
mod database;
mod user;
mod word;
mod response;
mod password_manager;
#[launch]
fn rocket() -> _ {
    dotenv().ok();
    rocket::build()
        .attach(Db::init()) 
        .mount("/", routes![])
        .attach(user::handlers::stage())
        .attach(word::handlers::stage())
}
