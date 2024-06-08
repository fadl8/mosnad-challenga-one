use crate::database::Db;

use diesel::sql_types::Integer;
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
    userId: i32
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


#[get("/user")]
async fn get_all_rows_by_userId(mut db: Connection<Db>) -> Result<Json<Vec<Word>>> { 
    // TODO :: get user id from access token 
    let user_id:i32 = 1;

    let words = words::table
        .select(words::all_columns)
        .filter(words::userId.eq(user_id))
        .load(&mut db)
        .await?;
    Ok(Json(words))
}

#[get("/admin")]
async fn get_all_rows_for_admin(mut db: Connection<Db>) -> Result<Json<Vec<Word>>> { 
    
    let words = words::table
        .select(words::all_columns)
        .filter(words::approved.eq(false))
        .load(&mut db)
        .await?;
    Ok(Json(words))
}

#[delete("/<id>")]
async fn delete_word(mut db: Connection<Db>, id: i32) -> Result<Json<String>, Status> {
    // Combine finding and deleting in a single query
    let deleted_word = match diesel::delete(words::table.filter(words::id.eq(id)))
        .get_result::<Word>(&mut *db)
        .await
    {
        Ok(word) => Some(word),
        Err(diesel::result::Error::NotFound) => None,
        Err(err) => {
            error!("Error deleting word: {}", err);
            return Err(Status::InternalServerError);
        }
    };

    match deleted_word {
        Some(word) => Ok(Json(format!("Word (ID: {}) deleted successfully", word.id))),
        None => Err(Status::NotFound),
    }
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
            routes![create_word, 
            get_all_rows, 
            search, 
            get_sorted, 
            get_by_id,
            get_all_rows_by_userId,
            get_all_rows_for_admin,
            delete_word],
        )
    })
}
