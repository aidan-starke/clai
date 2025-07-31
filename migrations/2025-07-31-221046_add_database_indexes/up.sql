-- Add indexes for frequently queried columns

-- Index for sessions.updated_at (used in get_last_session)
CREATE INDEX idx_sessions_updated_at ON sessions(updated_at DESC);

-- Index for sessions.display_name (used in get_session_by_name and list_named_sessions)
CREATE INDEX idx_sessions_display_name ON sessions(display_name);

-- Index for sessions.created_at (used in ordering operations)
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);

-- Index for messages.session_id (used in get_session_messages and FK lookups)
CREATE INDEX idx_messages_session_id ON messages(session_id);

-- Index for messages.created_at (used in message ordering)
CREATE INDEX idx_messages_created_at ON messages(created_at ASC);