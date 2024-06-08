use rocket_db_pools::diesel::PgPool;

#[derive(Database)]
#[database("diesel_postgres")]
pub struct Db(PgPool);
