use crate::db::ClaiDb;
use tracing::{error, info};

pub fn cleanup_old_sessions() {
    tokio::spawn(async {
        info!("Starting background database cleanup...");
        let mut db = ClaiDb::get();
        match db.cleanup_old_sessions() {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    info!("Cleaned up {} old database records", deleted_count);
                } else {
                    info!("No old sessions to clean up");
                }
            }
            Err(e) => {
                error!("Failed to cleanup old sessions: {}", e);
            }
        }
    });
}
