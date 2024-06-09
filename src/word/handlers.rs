use std::string;

use crate::{database::Db, jwt::Claims};

use diesel::sql_types::Integer;
use rocket::{
    fairing::AdHoc,
    http::Status,
    response::{status::Created, Debug},
    serde::{json::Json, Deserialize, Serialize},
};

use rocket_db_pools::Connection;

use rocket_db_pools::diesel::prelude::*;

use super::models::{words::{self, approved, id}, NewWord, Word};

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct NewWordView {
    id: i32,
    title: String,
    description: String,
    character: String,
    approved: bool,
    user_id: i32
}
 
// Getting random set of words
#[get("/?<page_index>&<limit>")]
async fn get_all_rows(mut db: Connection<Db> , page_index: i64, limit: i64) -> Result<Json<Vec<Word>>> {
    let words = words::table
        .select(words::all_columns)
        .limit(limit)
        .offset(10 * page_index)
        .load(&mut db)
        .await?;
    Ok(Json(words))
}

// Getting a list of all words sorted alphabetically
#[get("/get-sorted?<page_index>&<limit>")]
async fn get_sorted(mut db: Connection<Db> , page_index: i64, limit: i64) -> Result<Json<Vec<Word>>> {
    let words = words::table
        .select(words::all_columns)
        .order_by(words::character)
        .limit(limit)
        .offset(page_index * 10)
        .load(&mut db)
        .await?;
    Ok(Json(words))
}

// Getting a list of the characters which are the first character of a word
#[get("/get_characters_words")]
async fn get_characters_words(mut db: Connection<Db>) -> Result<Json<Vec<String>>> {

    let words = words::table
        .select(words::character)
        .distinct_on(words::character) 
        .load(&mut db)
        .await?;
    Ok(Json(words))
}

// Getting a list of the words by their first character
#[get("/get_words_by_characters/<character>?<page_index>&<limit>")]
async fn get_words_by_characters(mut db: Connection<Db>,character: String,  page_index: i64,limit: i64,) -> Result<Json<Vec<Word>>> {

    let words = words::table
        .select(words::all_columns)
        .filter(words::title.like(format!("{}%", character))) 
        .limit(limit)
        .offset(page_index * 10)
        .load(&mut db)
        .await?;
    Ok(Json(words))
}

// Getting a word by its Id
#[get("/<word_id>")]
async fn get_by_id(mut db: Connection<Db>, word_id: i32) -> Result<Json<Word>, Status> {
    let word = words::table.find(word_id).get_result::<Word>(&mut db).await; 
    match word {
        Ok(word) => Ok(Json(word)),
        Err(_) => Err(Status::NotFound),
    }
}

// Searching for all the available words stored in the database
#[get("/search/<search_word>?<page_index>&<limit>")]
async fn search(mut db: Connection<Db>, search_word: String,page_index: i64,limit: i64,) -> Result<Json<Vec<Word>>> {
    let words = words::table 
        .select(words::all_columns)
        .filter(words::title.eq(search_word))
        .filter(words::approved.eq(true))
        .limit(limit)
        .offset(page_index * 10)
        .load(&mut db)
        .await?;
    Ok(Json(words))
}

// Sending a word with its definition for admins to review
#[post("/", data = "<new_word>")]
async fn create_word(mut db: Connection<Db>, new_word: Json<NewWord>) -> Result<Created<Json<Word>>> {  
    let word = diesel::insert_into(words::table)
        .values(&*new_word)
        .returning(Word::as_returning())
        .get_result(&mut db)
        .await?;

    Ok(Created::new("/").body(Json(word)))
}

// Listing all the words created by the user
#[get("/user")]
async fn get_all_word_by_userId(mut db: Connection<Db>,claims:Claims) -> Result<Json<Vec<Word>>> { 
    // TODO :: get user id from access token 
    let words = words::table
        .select(words::all_columns)
        .filter(words::user_id.eq(claims.id))
        .load(&mut db)
        .await?;
    Ok(Json(words))
}


// Ability to delete a word created by the user
#[delete("/<word_id>")]
async fn delete_word(mut db: Connection<Db>,claims: Claims, word_id: i32) -> Result<Json<String>, Status> {
    if claims.is_admin {
        return Err(Status::Unauthorized); 
    }
    let deleted_word = match diesel::delete(words::table.filter(words::id.eq(word_id)))
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

// Getting a list of all the non approved word submitted by users
#[get("/admin?<page_index>&<limit>")]
async fn get_all_rows_for_admin(mut db: Connection<Db>, claims: Claims,page_index: i64,limit: i64,) -> Result<Json<Vec<Word>>, Status> { 
    
    if claims.is_admin {
        return Err(Status::Unauthorized);
    }

    let words = words::table
        .select(words::all_columns)
        .filter(words::approved.eq(false))
        .limit(limit)
        .offset(page_index * 10)
        .load(&mut db)
        .await
        .unwrap();

    Ok(Json(words))
}


// Ability to approve or reject non approved words
#[put("/update",data = "<new_word>")]
async fn update_word(mut db: Connection<Db>, claims: Claims,new_word: Json<NewWord>) -> Result<Json<Word>, Status> {
    
    if claims.is_admin {
        return Err(Status::Unauthorized);
    }

    let  word = words::table.find(new_word.id).get_result::<Word>(&mut db).await; 
    match word {
        Ok(mut word) => {
            let word_updated = diesel::update(words::table.filter(words::id.eq(new_word.id)))
             .set(approved.eq(new_word.approved))
             .execute(&mut db) ;
      
            Ok(Json(word))
        },
        Err(_) => Err(Status::NotFound),
    } 
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
            get_all_word_by_userId,
            get_all_rows_for_admin,
            get_characters_words,
            get_words_by_characters,
            update_word,
            delete_word],
        )
    })
}
