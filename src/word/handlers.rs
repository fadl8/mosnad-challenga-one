use crate::database::Db;

use rocket::{
    fairing::AdHoc,
    http::Status,
    response::{status::Created, Debug},
    serde::{json::Json, Deserialize, Serialize},
};

use rocket_db_pools::Connection;

use rocket_db_pools::diesel::prelude::*;

use super::models::{words, NewWord, Word};

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct NewWordView {
    title: String,
    description: String,
    character: String,
    approved: bool,
}

#[post("/", data = "<new_word>")]
async fn create_word(
    mut db: Connection<Db>,
    new_word: Json<NewWord>,
) -> Result<Created<Json<Word>>> { 
    let word = diesel::insert_into(words::table)
        .values(&*new_word)
        .returning(Word::as_returning())
        .get_result(&mut db)
        .await?;

    Ok(Created::new("/").body(Json(word)))
}

#[get("/")]
async fn get_all_rows(mut db: Connection<Db>) -> Result<Json<Vec<Word>>> {
    let words = words::table
        .select(words::all_columns)
        .load(&mut db)
        .await?;
    Ok(Json(words))
}

#[get("/<id>")]
async fn get_by_id(mut db: Connection<Db>, id: i32) -> Result<Json<Word>, Status> {
    let word = words::table.find(id).get_result::<Word>(&mut db).await; 
    match word {
        Ok(word) => Ok(Json(word)),
        Err(_) => Err(Status::NotFound),
    }
}


#[get("/search/<search_word>")]
async fn search(mut db: Connection<Db>, search_word: String) -> Result<Json<Vec<Word>>> {
    let words = words::table 
        .select(words::all_columns)
        .filter(words::title.eq(search_word))
        .filter(words::approved.eq(true))
        .load(&mut db)
        .await?;
    Ok(Json(words))
}

#[get("/get_sorted")]
async fn get_sorted(mut db: Connection<Db>) -> Result<Json<Vec<Word>>> {
    let words = words::table
        .select(words::all_columns)
        .order_by(words::character)
        .load(&mut db)
        .await?;
    Ok(Json(words))
}
 
pub fn stage() -> AdHoc {
    AdHoc::on_ignite("words endpoint", |rocket| async {
        rocket.mount(
            "/words",
            routes![create_word, get_all_rows, search, get_sorted, get_by_id],
        )
    })
}
