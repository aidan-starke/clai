use chrono::Utc;
use std::sync::OnceLock;

use entity::{messages, sessions};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set,
};

use crate::{
    config::Config,
    error::{ClaiError, Result},
};

static DB_INSTANCE: OnceLock<ClaiDb> = OnceLock::new();

#[derive(Clone)]
pub struct ClaiDb {
    connection: DatabaseConnection,
}

impl ClaiDb {
    pub async fn init() -> Result<()> {
        let config = Config::load()?;
        Self::init_with_url(&config.database_url).await
    }

    pub async fn init_with_url(database_url: &str) -> Result<()> {
        let connection = Database::connect(database_url).await.map_err(|e| {
            ClaiError::server(format!("Failed to connect to {}: {}", database_url, e))
        })?;

        Migrator::up(&connection, None)
            .await
            .map_err(|e| ClaiError::server(format!("Failed to run migrations: {}", e)))?;

        let db = ClaiDb { connection };

        if DB_INSTANCE.get().is_none() {
            DB_INSTANCE.set(db).map_err(|_| {
                ClaiError::server("Failed to initialize database instance".to_string())
            })?;
        }

        Ok(())
    }

    pub fn get() -> &'static ClaiDb {
        DB_INSTANCE.get().expect("Database not initialized")
    }

    // === SESSION CRUD OPERATIONS ===

    // CREATE
    pub async fn create_session(
        &self,
        name: &str,
        display_name: Option<&str>,
    ) -> Result<sessions::Model> {
        let now = Utc::now().naive_utc();
        let new_session = sessions::ActiveModel {
            name: Set(name.to_owned()),
            display_name: Set(display_name.map(|s| s.to_owned())),
            created_at: Set(now),
            updated_at: Set(now),
            role: Set(None),
            model: Set(None),
            ..Default::default()
        };

        let session = new_session
            .insert(&self.connection)
            .await
            .map_err(ClaiError::from)?;

        Ok(session)
    }

    // READ
    pub async fn get_last_session(&self) -> Result<sessions::Model> {
        let session = sessions::Entity::find()
            .order_by_desc(sessions::Column::UpdatedAt)
            .one(&self.connection)
            .await
            .map_err(ClaiError::from)?
            .ok_or_else(|| {
                ClaiError::Database(sea_orm::DbErr::RecordNotFound(
                    "No sessions found".to_string(),
                ))
            })?;

        Ok(session)
    }

    pub async fn get_session_by_name(&self, name: &str) -> Result<sessions::Model> {
        let session = sessions::Entity::find()
            .filter(sessions::Column::DisplayName.eq(name))
            .order_by_desc(sessions::Column::CreatedAt)
            .one(&self.connection)
            .await
            .map_err(ClaiError::from)?
            .ok_or_else(|| {
                ClaiError::Database(sea_orm::DbErr::RecordNotFound(format!(
                    "Session '{}' not found",
                    name
                )))
            })?;

        Ok(session)
    }

    pub async fn get_session_by_id(&self, session_id: i32) -> Result<sessions::Model> {
        let session = sessions::Entity::find_by_id(session_id)
            .one(&self.connection)
            .await
            .map_err(ClaiError::from)?
            .ok_or_else(|| {
                ClaiError::Database(sea_orm::DbErr::RecordNotFound(format!(
                    "Session {} not found",
                    session_id
                )))
            })?;

        Ok(session)
    }

    pub async fn list_named_sessions(&self) -> Result<Vec<sessions::Model>> {
        let sessions = sessions::Entity::find()
            .filter(sessions::Column::DisplayName.is_not_null())
            .order_by_desc(sessions::Column::CreatedAt)
            .all(&self.connection)
            .await
            .map_err(ClaiError::from)?;

        Ok(sessions)
    }

    // UPDATE
    pub async fn update_session_display_name(
        &self,
        session_id: i32,
        display_name: &str,
    ) -> Result<sessions::Model> {
        let session = sessions::Entity::find_by_id(session_id)
            .one(&self.connection)
            .await
            .map_err(ClaiError::from)?
            .ok_or_else(|| {
                ClaiError::Database(sea_orm::DbErr::RecordNotFound(format!(
                    "Session {} not found",
                    session_id
                )))
            })?;

        let mut session: sessions::ActiveModel = session.into();
        session.display_name = Set(Some(display_name.to_owned()));
        let updated_session = session
            .update(&self.connection)
            .await
            .map_err(ClaiError::from)?;

        Ok(updated_session)
    }

    pub async fn update_session_timestamp(&self, session_id: i32) -> Result<()> {
        let session = sessions::Entity::find_by_id(session_id)
            .one(&self.connection)
            .await
            .map_err(ClaiError::from)?
            .ok_or_else(|| {
                ClaiError::Database(sea_orm::DbErr::RecordNotFound(format!(
                    "Session {} not found",
                    session_id
                )))
            })?;

        let mut session: sessions::ActiveModel = session.into();
        session.updated_at = Set(Utc::now().naive_utc());
        session
            .update(&self.connection)
            .await
            .map_err(ClaiError::from)?;

        Ok(())
    }

    pub async fn update_session_role(
        &self,
        session_id: i32,
        role: Option<&str>,
    ) -> Result<sessions::Model> {
        let session = sessions::Entity::find_by_id(session_id)
            .one(&self.connection)
            .await
            .map_err(ClaiError::from)?
            .ok_or_else(|| {
                ClaiError::Database(sea_orm::DbErr::RecordNotFound(format!(
                    "Session {} not found",
                    session_id
                )))
            })?;

        let mut session: sessions::ActiveModel = session.into();
        session.role = Set(role.map(|s| s.to_owned()));
        let updated_session = session
            .update(&self.connection)
            .await
            .map_err(ClaiError::from)?;

        Ok(updated_session)
    }

    pub async fn update_session_model(
        &self,
        session_id: i32,
        model: &str,
    ) -> Result<sessions::Model> {
        let session = sessions::Entity::find_by_id(session_id)
            .one(&self.connection)
            .await
            .map_err(ClaiError::from)?
            .ok_or_else(|| {
                ClaiError::Database(sea_orm::DbErr::RecordNotFound(format!(
                    "Session {} not found",
                    session_id
                )))
            })?;

        let mut session: sessions::ActiveModel = session.into();
        session.model = Set(Some(model.to_owned()));
        let updated_session = session
            .update(&self.connection)
            .await
            .map_err(ClaiError::from)?;

        Ok(updated_session)
    }

    // DELETE
    pub async fn delete_session_by_name(&self, name: &str) -> Result<()> {
        // First get the session to verify it exists
        let session = self.get_session_by_name(name).await?;

        // Delete messages first (foreign key constraint)
        messages::Entity::delete_many()
            .filter(messages::Column::SessionId.eq(session.id))
            .exec(&self.connection)
            .await
            .map_err(ClaiError::from)?;

        // Then delete the session
        sessions::Entity::delete_by_id(session.id)
            .exec(&self.connection)
            .await
            .map_err(ClaiError::from)?;

        Ok(())
    }

    pub async fn cleanup_old_sessions(&self) -> Result<usize> {
        // Get the most recent session
        let latest_session = sessions::Entity::find()
            .order_by_desc(sessions::Column::CreatedAt)
            .one(&self.connection)
            .await
            .map_err(ClaiError::from)?;

        if let Some(keep_session) = latest_session {
            // Find all unnamed sessions except the most recent one
            let old_unnamed_sessions = sessions::Entity::find()
                .filter(sessions::Column::Id.ne(keep_session.id))
                .filter(sessions::Column::DisplayName.is_null())
                .all(&self.connection)
                .await
                .map_err(ClaiError::from)?;

            let old_session_ids: Vec<i32> = old_unnamed_sessions.iter().map(|s| s.id).collect();

            if old_session_ids.is_empty() {
                return Ok(0);
            }

            // Delete messages from old unnamed sessions first
            let deleted_messages = messages::Entity::delete_many()
                .filter(messages::Column::SessionId.is_in(old_session_ids.clone()))
                .exec(&self.connection)
                .await
                .map_err(ClaiError::from)?;

            // Delete old unnamed sessions
            let deleted_sessions = sessions::Entity::delete_many()
                .filter(sessions::Column::Id.is_in(old_session_ids))
                .exec(&self.connection)
                .await
                .map_err(ClaiError::from)?;

            Ok((deleted_messages.rows_affected + deleted_sessions.rows_affected) as usize)
        } else {
            // No sessions to clean up
            Ok(0)
        }
    }

    // === MESSAGE CRUD OPERATIONS ===

    // CREATE
    pub async fn create_message(
        &self,
        session_id: i32,
        role: &str,
        content: &str,
    ) -> Result<messages::Model> {
        let new_message = messages::ActiveModel {
            session_id: Set(session_id),
            role: Set(role.to_owned()),
            content: Set(content.to_owned()),
            created_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        let message = new_message
            .insert(&self.connection)
            .await
            .map_err(ClaiError::from)?;
        Ok(message)
    }

    // READ
    pub async fn get_session_messages(&self, session_id: i32) -> Result<Vec<messages::Model>> {
        let messages = messages::Entity::find()
            .filter(messages::Column::SessionId.eq(session_id))
            .order_by_asc(messages::Column::CreatedAt)
            .all(&self.connection)
            .await
            .map_err(ClaiError::from)?;

        Ok(messages)
    }
}
