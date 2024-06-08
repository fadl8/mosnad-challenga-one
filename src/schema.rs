// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        password -> Varchar,
        is_admin -> Bool,
    }
}

diesel::table! {
    words (id) {
        id -> Int4,
        title -> Varchar,
        description -> Varchar,
        character -> Varchar,
        approved -> Bool,
        user_id -> Nullable<Int4>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    users,
    words,
);
