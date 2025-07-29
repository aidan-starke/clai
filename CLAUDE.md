# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CLAI (Command Line Artificial Interface) is a Rust-based CLI chat application for interacting with Claude AI. It features a client-server architecture where the CLI client communicates with a local HTTP server that handles persistence and AI integration.

## Development Commands

```bash
# Primary development
cargo run                    # Run client (auto-starts server on port 3500)
cargo run -- --server        # Run server only
cargo check                  # Type check without building
cargo fmt                    # Format code

# Database management
diesel migration run         # Apply pending migrations
diesel setup                 # Initialize database from scratch
diesel migration generate NAME  # Create new migration

# Environment setup
export ANTHROPIC_API_KEY=sk-...  # Required for Claude AI integration
export DATABASE_URL=clai.db      # Optional (defaults to clai.db)
export CLAI_SERVER_URL=http://localhost:3500  # Optional server URL
```

## Architecture Overview

### Client-Server Design

- **Client**: Interactive CLI with session management and chat interface
- **Server**: HTTP API server on port 3500 with SQLite persistence and Claude AI integration
- **Auto-startup**: Client automatically starts server if not running

### Core Modules

**`src/session_manager.rs`**: HTTP client managing session state with `Cell<Option<i32>>` for current session ID. All methods use internal session tracking - never pass session IDs as parameters.

**`src/commands.rs`**: Slash command processor handling 7 commands (`/clear`, `/new`, `/save`, `/delete`, `/list`, `/resume`, `/role`). Commands return `anyhow::Result<()>` and update session state internally.

**`src/server/handlers/`**: REST API handlers for session CRUD operations and chat integration with Anthropic Claude API.

**`src/db/lib.rs`**: Singleton database access using `OnceLock<Mutex<ClaiDb>>` pattern. Access via `ClaiDb::get()` returns `MutexGuard`.

**`src/types.rs`**: Central location for all API types used by both client and server (`SessionResponse`, `ChatRequest`, etc.). Import with `use crate::types::*`.

### Key Patterns

**Database Access**: Always use `let mut db = ClaiDb::get()` for database operations. Migrations run automatically on startup.

**Session Management**: SessionManager handles current session internally via `Cell`. Commands like `/new` and `/resume` update internal state automatically.

**Type Organization**: All shared API types live in `src/types.rs`. Handler-specific types (like `ClaudeRequest`) remain in their respective handlers.

**Error Handling**: Use `handle_db_operation!` macro for database operations. Server handlers return `Result<JsonResponse<T>, StatusCode>`.

## Database Schema

- **sessions**: `id`, `name`, `display_name`, `created_at`, `updated_at`, `role`
- **messages**: `id`, `session_id`, `role` (user/assistant), `content`, `created_at`

Sessions support role-playing where Claude adopts different personas. Unnamed sessions are auto-cleaned keeping only the most recent.

## Terminal Interface

Custom terminal UI with:

- Command autocomplete dropdown for slash commands
- Real-time input using `console::Term::read_key()`
- Progress spinners during AI responses
- Rich formatting with colors and styling

## Build Requirements

- **Rust toolchain**: Nightly (required for `iter_map_windows` feature)
- **Diesel CLI**: For database migrations
- **SQLite**: Database backend
- **Anthropic API key**: For Claude AI integration

## Code Conventions

- Use `anyhow::Result` for error handling
- Import types with `use crate::types::*`
- Database operations via singleton pattern

