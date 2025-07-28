use crate::db::schema::{messages, sessions};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = sessions)]
pub struct Session {
    pub id: i32,
    pub name: String,
    pub display_name: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub role: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = sessions)]
pub struct NewSession<'a> {
    pub name: &'a str,
    pub display_name: Option<&'a str>,
    pub role: Option<&'a str>,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = messages)]
pub struct Message {
    pub id: i32,
    pub session_id: i32,
    pub role: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = messages)]
pub struct NewMessage<'a> {
    pub session_id: i32,
    pub role: &'a str,
    pub content: &'a str,
}
