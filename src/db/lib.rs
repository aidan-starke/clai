use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::{
    config::Config,
    db::{
        models::{Message, NewMessage, NewSession, Session},
        schema::{messages, sessions},
    },
    error::{ClaiError, Result},
};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct ClaiDb;

impl ClaiDb {
    pub fn init() -> Result<()> {
        let mut connection = Self::establish_connection()?;
        connection
            .run_pending_migrations(MIGRATIONS)
            .map_err(|e| ClaiError::server(format!("Failed to run migrations: {}", e)))?;
        Ok(())
    }

    fn establish_connection() -> Result<SqliteConnection> {
        let config = Config::load()?;
        SqliteConnection::establish(&config.database_url).map_err(|e| {
            ClaiError::server(format!(
                "Failed to connect to {}: {}",
                config.database_url, e
            ))
        })
    }

    fn with_connection<F, R>(f: F) -> Result<R>
    where
        F: FnOnce(&mut SqliteConnection) -> diesel::result::QueryResult<R>,
    {
        let mut connection = Self::establish_connection()?;
        f(&mut connection).map_err(ClaiError::Database)
    }

    // === SESSION CRUD OPERATIONS ===

    // CREATE
    pub fn create_session(name: &str, display_name: Option<&str>) -> Result<Session> {
        Self::with_connection(|conn| {
            let new_session = NewSession {
                name,
                display_name,
                role: None,
                model: None,
            };

            diesel::insert_into(sessions::table)
                .values(&new_session)
                .execute(conn)?;

            sessions::table.order(sessions::id.desc()).first(conn)
        })
    }

    // READ
    pub fn get_last_session() -> Result<Session> {
        Self::with_connection(|conn| {
            sessions::table
                .order(sessions::updated_at.desc())
                .first(conn)
        })
    }

    pub fn get_session_by_name(name: &str) -> Result<Session> {
        Self::with_connection(|conn| {
            sessions::table
                .filter(sessions::display_name.eq(name))
                .order(sessions::created_at.desc())
                .first(conn)
        })
    }

    pub fn get_session_by_id(session_id: i32) -> Result<Session> {
        Self::with_connection(|conn| sessions::table.find(session_id).first(conn))
    }

    pub fn list_named_sessions() -> Result<Vec<Session>> {
        Self::with_connection(|conn| {
            sessions::table
                .filter(sessions::display_name.is_not_null())
                .order(sessions::created_at.desc())
                .load(conn)
        })
    }

    // UPDATE
    pub fn update_session_display_name(session_id: i32, display_name: &str) -> Result<Session> {
        Self::with_connection(|conn| {
            diesel::update(sessions::table.find(session_id))
                .set(sessions::display_name.eq(display_name))
                .execute(conn)?;

            sessions::table.find(session_id).first(conn)
        })
    }

    pub fn update_session_timestamp(session_id: i32) -> Result<()> {
        use chrono::Utc;

        Self::with_connection(|conn| {
            diesel::update(sessions::table.find(session_id))
                .set(sessions::updated_at.eq(Utc::now().naive_utc()))
                .execute(conn)?;

            Ok(())
        })
    }

    pub fn update_session_role(session_id: i32, role: Option<&str>) -> Result<Session> {
        Self::with_connection(|conn| {
            diesel::update(sessions::table.find(session_id))
                .set(sessions::role.eq(role))
                .execute(conn)?;

            sessions::table.find(session_id).first(conn)
        })
    }

    pub fn update_session_model(session_id: i32, model: &str) -> Result<Session> {
        Self::with_connection(|conn| {
            diesel::update(sessions::table.find(session_id))
                .set(sessions::model.eq(model))
                .execute(conn)?;

            sessions::table.find(session_id).first(conn)
        })
    }

    // DELETE
    pub fn delete_session_by_name(name: &str) -> Result<()> {
        Self::with_connection(|conn| {
            // First get the session to verify it exists
            let session = sessions::table
                .filter(sessions::display_name.eq(name))
                .order(sessions::created_at.desc())
                .first::<Session>(conn)?;

            // Delete messages first (foreign key constraint)
            diesel::delete(messages::table)
                .filter(messages::session_id.eq(session.id))
                .execute(conn)?;

            // Then delete the session
            diesel::delete(sessions::table)
                .filter(sessions::id.eq(session.id))
                .execute(conn)?;

            Ok(())
        })
    }

    pub fn cleanup_old_sessions() -> Result<usize> {
        Self::with_connection(|conn| {
            // Get the ID of the most recent session
            let latest_session_id: Option<i32> = sessions::table
                .select(sessions::id)
                .order(sessions::created_at.desc())
                .first(conn)
                .optional()?;

            if let Some(keep_session_id) = latest_session_id {
                // Delete messages from old unnamed sessions first (foreign key constraint)
                // Only delete sessions that don't have a display_name (are not saved) and are not the most recent
                let deleted_messages = diesel::delete(messages::table)
                    .filter(
                        messages::session_id.ne(keep_session_id).and(
                            messages::session_id.eq_any(
                                sessions::table
                                    .select(sessions::id)
                                    .filter(sessions::display_name.is_null()),
                            ),
                        ),
                    )
                    .execute(conn)?;

                // Delete old unnamed sessions (preserve named sessions)
                let deleted_sessions = diesel::delete(sessions::table)
                    .filter(
                        sessions::id
                            .ne(keep_session_id)
                            .and(sessions::display_name.is_null()),
                    )
                    .execute(conn)?;

                Ok(deleted_messages + deleted_sessions)
            } else {
                // No sessions to clean up
                Ok(0)
            }
        })
    }

    // === MESSAGE CRUD OPERATIONS ===

    // CREATE
    pub fn create_message(session_id: i32, role: &str, content: &str) -> Result<Message> {
        Self::with_connection(|conn| {
            let new_message = NewMessage {
                session_id,
                role,
                content,
            };

            diesel::insert_into(messages::table)
                .values(&new_message)
                .execute(conn)?;

            messages::table.order(messages::id.desc()).first(conn)
        })
    }

    // READ
    pub fn get_session_messages(session_id: i32) -> Result<Vec<Message>> {
        Self::with_connection(|conn| {
            messages::table
                .filter(messages::session_id.eq(session_id))
                .order(messages::created_at.asc())
                .load(conn)
        })
    }
}
