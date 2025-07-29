// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Integer,
        session_id -> Integer,
        role -> Text,
        content -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    sessions (id) {
        id -> Integer,
        name -> Text,
        display_name -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        role -> Nullable<Text>,
    }
}

diesel::joinable!(messages -> sessions (session_id));

diesel::allow_tables_to_appear_in_same_query!(messages, sessions,);
