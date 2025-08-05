use chrono::Utc;
use entity::{messages, sessions};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set,
};
use tempfile::NamedTempFile;

struct TestDb {
    connection: DatabaseConnection,
    _temp_file: NamedTempFile,
}

impl TestDb {
    async fn new() -> Self {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let database_url = format!("sqlite://{}", temp_file.path().to_string_lossy());

        let connection = Database::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        Migrator::up(&connection, None)
            .await
            .expect("Failed to run migrations");

        Self {
            connection,
            _temp_file: temp_file,
        }
    }
}

#[tokio::test]
async fn test_create_session() {
    let test_db = TestDb::new().await;

    let now = Utc::now().naive_utc();
    let new_session = sessions::ActiveModel {
        name: Set("test_session".to_owned()),
        display_name: Set(Some("Test Session".to_owned())),
        created_at: Set(now),
        updated_at: Set(now),
        role: Set(None),
        model: Set(None),
        ..Default::default()
    };

    let session = new_session
        .insert(&test_db.connection)
        .await
        .expect("Failed to create session");

    assert_eq!(session.name, "test_session");
    assert_eq!(session.display_name, Some("Test Session".to_string()));
    assert!(session.id > 0);
}

#[tokio::test]
async fn test_find_session_by_id() {
    let test_db = TestDb::new().await;

    let now = Utc::now().naive_utc();
    let new_session = sessions::ActiveModel {
        name: Set("test_session".to_owned()),
        display_name: Set(Some("Test Session".to_owned())),
        created_at: Set(now),
        updated_at: Set(now),
        role: Set(None),
        model: Set(None),
        ..Default::default()
    };

    let created_session = new_session
        .insert(&test_db.connection)
        .await
        .expect("Failed to create session");

    let found_session = sessions::Entity::find_by_id(created_session.id)
        .one(&test_db.connection)
        .await
        .expect("Failed to query session")
        .expect("Session not found");

    assert_eq!(found_session.name, "test_session");
    assert_eq!(found_session.display_name, Some("Test Session".to_string()));
}

#[tokio::test]
async fn test_find_session_by_display_name() {
    let test_db = TestDb::new().await;

    let now = Utc::now().naive_utc();
    let new_session = sessions::ActiveModel {
        name: Set("test_session".to_owned()),
        display_name: Set(Some("Named Session".to_owned())),
        created_at: Set(now),
        updated_at: Set(now),
        role: Set(None),
        model: Set(None),
        ..Default::default()
    };

    let _created_session = new_session
        .insert(&test_db.connection)
        .await
        .expect("Failed to create session");

    let found_session = sessions::Entity::find()
        .filter(sessions::Column::DisplayName.eq("Named Session"))
        .one(&test_db.connection)
        .await
        .expect("Failed to query session")
        .expect("Session not found");

    assert_eq!(
        found_session.display_name,
        Some("Named Session".to_string())
    );
}

#[tokio::test]
async fn test_list_named_sessions() {
    let test_db = TestDb::new().await;

    let now = Utc::now().naive_utc();

    // Create unnamed session
    let unnamed_session = sessions::ActiveModel {
        name: Set("unnamed".to_owned()),
        display_name: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        role: Set(None),
        model: Set(None),
        ..Default::default()
    };

    let _unnamed = unnamed_session
        .insert(&test_db.connection)
        .await
        .expect("Failed to create unnamed session");

    // Create named sessions
    let named_session1 = sessions::ActiveModel {
        name: Set("named1".to_owned()),
        display_name: Set(Some("Named Session 1".to_owned())),
        created_at: Set(now),
        updated_at: Set(now),
        role: Set(None),
        model: Set(None),
        ..Default::default()
    };

    let _named1 = named_session1
        .insert(&test_db.connection)
        .await
        .expect("Failed to create named session 1");

    let named_session2 = sessions::ActiveModel {
        name: Set("named2".to_owned()),
        display_name: Set(Some("Named Session 2".to_owned())),
        created_at: Set(now),
        updated_at: Set(now),
        role: Set(None),
        model: Set(None),
        ..Default::default()
    };

    let _named2 = named_session2
        .insert(&test_db.connection)
        .await
        .expect("Failed to create named session 2");

    let named_sessions = sessions::Entity::find()
        .filter(sessions::Column::DisplayName.is_not_null())
        .all(&test_db.connection)
        .await
        .expect("Failed to list named sessions");

    assert_eq!(named_sessions.len(), 2);
    assert!(named_sessions.iter().all(|s| s.display_name.is_some()));
}

#[tokio::test]
async fn test_create_message() {
    let test_db = TestDb::new().await;

    // First create a session
    let now = Utc::now().naive_utc();
    let new_session = sessions::ActiveModel {
        name: Set("test_session".to_owned()),
        display_name: Set(Some("Test Session".to_owned())),
        created_at: Set(now),
        updated_at: Set(now),
        role: Set(None),
        model: Set(None),
        ..Default::default()
    };

    let session = new_session
        .insert(&test_db.connection)
        .await
        .expect("Failed to create session");

    // Create a message
    let new_message = messages::ActiveModel {
        session_id: Set(session.id),
        role: Set("user".to_owned()),
        content: Set("Hello, world!".to_owned()),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    let message = new_message
        .insert(&test_db.connection)
        .await
        .expect("Failed to create message");

    assert_eq!(message.session_id, session.id);
    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Hello, world!");
    assert!(message.id > 0);
}

#[tokio::test]
async fn test_get_session_messages() {
    let test_db = TestDb::new().await;

    // Create a session
    let now = Utc::now().naive_utc();
    let new_session = sessions::ActiveModel {
        name: Set("test_session".to_owned()),
        display_name: Set(Some("Test Session".to_owned())),
        created_at: Set(now),
        updated_at: Set(now),
        role: Set(None),
        model: Set(None),
        ..Default::default()
    };

    let session = new_session
        .insert(&test_db.connection)
        .await
        .expect("Failed to create session");

    // Create messages
    let msg1 = messages::ActiveModel {
        session_id: Set(session.id),
        role: Set("user".to_owned()),
        content: Set("First message".to_owned()),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    let _message1 = msg1
        .insert(&test_db.connection)
        .await
        .expect("Failed to create first message");

    let msg2 = messages::ActiveModel {
        session_id: Set(session.id),
        role: Set("assistant".to_owned()),
        content: Set("Second message".to_owned()),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    let _message2 = msg2
        .insert(&test_db.connection)
        .await
        .expect("Failed to create second message");

    // Query messages
    let messages = messages::Entity::find()
        .filter(messages::Column::SessionId.eq(session.id))
        .order_by_asc(messages::Column::CreatedAt)
        .all(&test_db.connection)
        .await
        .expect("Failed to get session messages");

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].content, "First message");
    assert_eq!(messages[1].content, "Second message");
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[1].role, "assistant");
}

#[tokio::test]
async fn test_update_session() {
    let test_db = TestDb::new().await;

    // Create a session
    let now = Utc::now().naive_utc();
    let new_session = sessions::ActiveModel {
        name: Set("test_session".to_owned()),
        display_name: Set(Some("Original Name".to_owned())),
        created_at: Set(now),
        updated_at: Set(now),
        role: Set(None),
        model: Set(None),
        ..Default::default()
    };

    let session = new_session
        .insert(&test_db.connection)
        .await
        .expect("Failed to create session");

    // Update the session
    let mut session_active: sessions::ActiveModel = session.into();
    session_active.display_name = Set(Some("Updated Name".to_owned()));
    session_active.role = Set(Some("assistant".to_owned()));

    let updated_session = session_active
        .update(&test_db.connection)
        .await
        .expect("Failed to update session");

    assert_eq!(
        updated_session.display_name,
        Some("Updated Name".to_string())
    );
    assert_eq!(updated_session.role, Some("assistant".to_string()));
}

#[tokio::test]
async fn test_delete_session() {
    let test_db = TestDb::new().await;

    // Create a session
    let now = Utc::now().naive_utc();
    let new_session = sessions::ActiveModel {
        name: Set("test_session".to_owned()),
        display_name: Set(Some("To Delete".to_owned())),
        created_at: Set(now),
        updated_at: Set(now),
        role: Set(None),
        model: Set(None),
        ..Default::default()
    };

    let session = new_session
        .insert(&test_db.connection)
        .await
        .expect("Failed to create session");

    // Delete the session
    sessions::Entity::delete_by_id(session.id)
        .exec(&test_db.connection)
        .await
        .expect("Failed to delete session");

    // Verify it's gone
    let result = sessions::Entity::find_by_id(session.id)
        .one(&test_db.connection)
        .await
        .expect("Failed to query for deleted session");

    assert!(result.is_none());
}


