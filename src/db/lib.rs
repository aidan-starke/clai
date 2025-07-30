use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::db::models::{Message, NewMessage, NewSession, Session};
use crate::db::schema::{messages, sessions};

static DATABASE: OnceLock<Mutex<ClaiDb>> = OnceLock::new();

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct ClaiDb {
    connection: SqliteConnection,
}

impl ClaiDb {
    fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "clai.db".to_string());
        let mut connection = SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        // Run migrations
        connection
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        ClaiDb { connection }
    }

    pub fn get() -> MutexGuard<'static, ClaiDb> {
        DATABASE
            .get_or_init(|| Mutex::new(ClaiDb::new()))
            .lock()
            .unwrap()
    }

    fn connection(&mut self) -> &mut SqliteConnection {
        &mut self.connection
    }

    // === SESSION CRUD OPERATIONS ===

    // CREATE
    pub fn create_session(
        &mut self,
        name: &str,
        display_name: Option<&str>,
    ) -> QueryResult<Session> {
        let conn = self.connection();
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
    }

    // READ
    pub fn get_last_session(&mut self) -> QueryResult<Session> {
        sessions::table
            .order(sessions::updated_at.desc())
            .first(self.connection())
    }

    pub fn get_session_by_name(&mut self, name: &str) -> QueryResult<Session> {
        sessions::table
            .filter(sessions::display_name.eq(name))
            .order(sessions::created_at.desc())
            .first(self.connection())
    }

    pub fn get_session_by_id(&mut self, session_id: i32) -> QueryResult<Session> {
        sessions::table.find(session_id).first(self.connection())
    }

    pub fn list_named_sessions(&mut self) -> QueryResult<Vec<Session>> {
        sessions::table
            .filter(sessions::display_name.is_not_null())
            .order(sessions::created_at.desc())
            .load(self.connection())
    }

    // UPDATE
    pub fn update_session_display_name(
        &mut self,
        session_id: i32,
        display_name: &str,
    ) -> QueryResult<Session> {
        let conn = self.connection();
        diesel::update(sessions::table.find(session_id))
            .set(sessions::display_name.eq(display_name))
            .execute(conn)?;

        sessions::table.find(session_id).first(conn)
    }

    pub fn update_session_timestamp(&mut self, session_id: i32) -> QueryResult<()> {
        use chrono::Utc;

        diesel::update(sessions::table.find(session_id))
            .set(sessions::updated_at.eq(Utc::now().naive_utc()))
            .execute(self.connection())?;

        Ok(())
    }

    pub fn update_session_role(
        &mut self,
        session_id: i32,
        role: Option<&str>,
    ) -> QueryResult<Session> {
        let conn = self.connection();
        diesel::update(sessions::table.find(session_id))
            .set(sessions::role.eq(role))
            .execute(conn)?;

        sessions::table.find(session_id).first(conn)
    }

    pub fn update_session_model(
        &mut self,
        session_id: i32,
        model: &str,
    ) -> QueryResult<Session> {
        let conn = self.connection();
        diesel::update(sessions::table.find(session_id))
            .set(sessions::model.eq(model))
            .execute(conn)?;

        sessions::table.find(session_id).first(conn)
    }

    // DELETE
    pub fn delete_session_by_name(&mut self, name: &str) -> QueryResult<()> {
        let conn = self.connection();

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
    }

    pub fn cleanup_old_sessions(&mut self) -> QueryResult<usize> {
        let conn = self.connection();

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
    }

    // === MESSAGE CRUD OPERATIONS ===

    // CREATE
    pub fn create_message(
        &mut self,
        session_id: i32,
        role: &str,
        content: &str,
    ) -> QueryResult<Message> {
        let conn = self.connection();
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

    // READ
    pub fn get_session_messages(&mut self, session_id: i32) -> QueryResult<Vec<Message>> {
        messages::table
            .filter(messages::session_id.eq(session_id))
            .order(messages::created_at.asc())
            .load(self.connection())
    }
}
