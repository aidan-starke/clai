use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::db::models::{Message, NewMessage, NewSession, Session};
use crate::db::schema::{messages, sessions};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn establish_connection() -> SqliteConnection {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "clai.db".to_string());

    let mut connection = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    // Run migrations
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");

    connection
}

pub fn create_session(
    conn: &mut SqliteConnection,
    name: &str,
    display_name: Option<&str>,
) -> QueryResult<Session> {
    let new_session = NewSession { name, display_name };

    diesel::insert_into(sessions::table)
        .values(&new_session)
        .execute(conn)?;

    sessions::table.order(sessions::id.desc()).first(conn)
}

pub fn update_session_display_name(
    conn: &mut SqliteConnection,
    session_id: i32,
    display_name: &str,
) -> QueryResult<Session> {
    diesel::update(sessions::table.find(session_id))
        .set(sessions::display_name.eq(display_name))
        .execute(conn)?;

    sessions::table.find(session_id).first(conn)
}

pub fn get_last_session(conn: &mut SqliteConnection) -> QueryResult<Session> {
    sessions::table
        .order(sessions::created_at.desc())
        .first(conn)
}

pub fn get_session_by_name(conn: &mut SqliteConnection, name: &str) -> QueryResult<Session> {
    sessions::table
        .filter(sessions::display_name.eq(name))
        .order(sessions::created_at.desc())
        .first(conn)
}

pub fn list_named_sessions(conn: &mut SqliteConnection) -> QueryResult<Vec<Session>> {
    sessions::table
        .filter(sessions::display_name.is_not_null())
        .order(sessions::created_at.desc())
        .load(conn)
}

pub fn create_message(
    conn: &mut SqliteConnection,
    session_id: i32,
    role: &str,
    content: &str,
) -> QueryResult<Message> {
    let new_message = NewMessage {
        session_id,
        role,
        content,
    };

    diesel::insert_into(messages::table)
        .values(&new_message)
        .execute(conn)?;

    messages::table.order(messages::id.desc()).first(conn)
}

pub fn get_session_messages(
    conn: &mut SqliteConnection,
    session_id: i32,
) -> QueryResult<Vec<Message>> {
    messages::table
        .filter(messages::session_id.eq(session_id))
        .order(messages::created_at.asc())
        .load(conn)
}

pub fn cleanup_old_sessions(conn: &mut SqliteConnection) -> QueryResult<usize> {
    // Get the ID of the most recent session
    let latest_session_id: Option<i32> = sessions::table
        .select(sessions::id)
        .order(sessions::created_at.desc())
        .first(conn)
        .optional()?;

    if let Some(keep_session_id) = latest_session_id {
        // Delete messages from old sessions first (foreign key constraint)
        let deleted_messages = diesel::delete(messages::table)
            .filter(messages::session_id.ne(keep_session_id))
            .execute(conn)?;

        // Delete old sessions
        let deleted_sessions = diesel::delete(sessions::table)
            .filter(sessions::id.ne(keep_session_id))
            .execute(conn)?;

        Ok(deleted_messages + deleted_sessions)
    } else {
        // No sessions to clean up
        Ok(0)
    }
}
