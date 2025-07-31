use clai::db::ClaiDb;
use test_macros::db_test;

mod common;

#[db_test]
fn test_session_crud_operations() {
    let session = ClaiDb::create_session("test-session", Some("Test Session"))
        .expect("Should create session");

    assert_eq!(session.name, "test-session");
    assert_eq!(session.display_name, Some("Test Session".to_string()));
    assert!(session.id > 0);

    let found_session = ClaiDb::get_session_by_id(session.id).expect("Should find session by ID");

    assert_eq!(found_session.id, session.id);
    assert_eq!(found_session.name, "test-session");

    let found_by_name =
        ClaiDb::get_session_by_name("Test Session").expect("Should find session by display name");

    assert_eq!(found_by_name.id, session.id);

    let updated = ClaiDb::update_session_display_name(session.id, "Updated Session")
        .expect("Should update display name");

    assert_eq!(updated.display_name, Some("Updated Session".to_string()));

    let with_role =
        ClaiDb::update_session_role(session.id, Some("assistant")).expect("Should update role");

    assert_eq!(with_role.role, Some("assistant".to_string()));

    let with_model =
        ClaiDb::update_session_model(session.id, "claude-3-sonnet").expect("Should update model");

    assert_eq!(with_model.model, Some("claude-3-sonnet".to_string()));

    ClaiDb::delete_session_by_name("Updated Session").expect("Should delete session");

    let deleted_result = ClaiDb::get_session_by_id(session.id);
    assert!(deleted_result.is_err(), "Session should be deleted");
}

#[db_test]
fn test_message_operations() {
    let session = ClaiDb::create_session("msg-test", None).expect("Should create session");

    let user_msg =
        ClaiDb::create_message(session.id, "user", "Hello!").expect("Should create user message");

    assert_eq!(user_msg.session_id, session.id);
    assert_eq!(user_msg.role, "user");
    assert_eq!(user_msg.content, "Hello!");

    let _ = ClaiDb::create_message(session.id, "assistant", "Hi there!")
        .expect("Should create assistant message");

    let messages = ClaiDb::get_session_messages(session.id).expect("Should get session messages");

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[0].content, "Hello!");
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(messages[1].content, "Hi there!");
}

#[db_test]
fn test_session_listing_and_cleanup() {
    let _ = ClaiDb::create_session("temp1", None).expect("Should create temp session 1");
    let _ = ClaiDb::create_session("temp2", Some("Named Session"))
        .expect("Should create named session");
    let _ = ClaiDb::create_session("temp3", None).expect("Should create temp session 3");

    let named_sessions = ClaiDb::list_named_sessions().expect("Should list named sessions");

    assert_eq!(named_sessions.len(), 1);
    assert_eq!(
        named_sessions[0].display_name,
        Some("Named Session".to_string())
    );

    let cleanup_count = ClaiDb::cleanup_old_sessions().expect("Should cleanup old sessions");

    assert!(cleanup_count > 0);

    let named_still_exists = ClaiDb::get_session_by_name("Named Session");
    assert!(
        named_still_exists.is_ok(),
        "Named session should not be cleaned up"
    );
}

#[db_test]
fn test_get_last_session() {
    let no_sessions = ClaiDb::get_last_session();
    assert!(no_sessions.is_err(), "Should fail when no sessions exist");

    let session1 = ClaiDb::create_session("first", None).expect("Should create first session");

    let last = ClaiDb::get_last_session().expect("Should get last session");
    assert_eq!(last.id, session1.id);

    let session2 = ClaiDb::create_session("second", None).expect("Should create second session");

    ClaiDb::update_session_timestamp(session2.id).expect("Should update session2 timestamp");

    let last = ClaiDb::get_last_session().expect("Should get last session");
    assert_eq!(
        last.id, session2.id,
        "session2 should be last after being accessed"
    );

    ClaiDb::update_session_timestamp(session1.id).expect("Should update session1 timestamp");

    let last = ClaiDb::get_last_session().expect("Should get last session");
    assert_eq!(
        last.id, session1.id,
        "session1 should be last after being accessed again"
    );
}

#[db_test]
fn test_update_session_timestamp() {
    let session = ClaiDb::create_session("timestamp-test", None).expect("Should create session");

    let original_updated_at = session.updated_at;

    std::thread::sleep(std::time::Duration::from_millis(10));

    ClaiDb::update_session_timestamp(session.id).expect("Should update timestamp");

    let updated_session =
        ClaiDb::get_session_by_id(session.id).expect("Should get updated session");

    assert!(
        updated_session.updated_at > original_updated_at,
        "Timestamp should be updated"
    );
}
