use crate::{
    database::Db,
    jwt::{self, Claims},
    password_manager,
};

use rocket::{
    fairing::AdHoc,
    response::{
        status::{BadRequest, Created, NotFound},
        Debug,
    },
    serde::{json::Json, Deserialize, Serialize},
};

use rocket_db_pools::{Connection, Database};

use rocket_db_pools::diesel::prelude::*;

use super::models::{users, NewUser, User};

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct UserLoginView {
    email: String,
    password: String,
    token: Option<String>,
}

#[post("/signup", data = "<new_user>")]
async fn singup(
    mut db: Connection<Db>,
    mut new_user: Json<NewUser>,
) -> Result<Created<Json<User>>, BadRequest<String>> {
    match password_manager::hash_password(&new_user.password) {
        Ok(password) => new_user.password = password,
        Err(_) => return Err(BadRequest("bad password".to_string())),
    };

    let user = diesel::insert_into(users::table)
        .values(&*new_user)
        .returning(User::as_returning())
        .get_result(&mut db)
        .await;

    match user {
        Ok(user) => Ok(Created::new("/").body(Json(user))),
        Err(_) => return Err(BadRequest("could not create new user".to_string())),
    }
}

#[get("/login", data = "<user_view>")]
async fn login<'a>(
    mut db: Connection<Db>,
    mut user_view: Json<UserLoginView>,
) -> Result<Created<Json<UserLoginView>>, NotFound<&'a str>> {
    let user = users::table
        .select(users::all_columns)
        .filter(users::email.eq(&user_view.email))
        .first::<User>(&mut db)
        .await;

    match user {
        Ok(user) => {
            let is_valid = password_manager::verify_password(&user.password, &user_view.password);
            if let Err(_) = is_valid {
                return Err(NotFound("invalid  email or password"));
            }
            match jwt::create_jwt(Claims::new(user.id, user.is_admin)) {
                Ok(token) => {
                    user_view.token = Some(token);
                    Ok(Created::new("/").body(user_view))
                }
                Err(_) => Err(NotFound("invalid  email or password")),
            }
        }
        Err(_) => Err(NotFound("invalid  email or password")),
    }
}

#[get("/")]
async fn get_all_user(mut db: Connection<Db>) -> Result<Json<Vec<User>>> { 
    
    let userList = users::table
        .select(users::all_columns) 
        .load(&mut db)
        .await?;
    
    Ok(Json(userList))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("users endpoint", |rocket| async {
        rocket
             .mount("/users", routes![singup, login])
    })
}
