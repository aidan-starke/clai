# CLAI - Command Line Artificial Interface

A production-quality Rust CLI application for conversational interaction with Claude AI, featuring a client-server architecture, persistent chat sessions, and MCP (Model Context Protocol) integration.

## Project Overview

CLAI is a fully-featured command-line chat client that I built independently to explore Rust's async ecosystem, database ORMs, HTTP frameworks, and real-world API integration. The project demonstrates modern Rust patterns and production-ready architecture with clean separation of concerns across a workspace of six crates.

## Features

### Core Functionality

- **Interactive Chat**: Real-time conversational interface with Claude AI
- **Session Management**: Create, save, resume, and delete chat sessions with SQLite persistence
- **Role-Playing**: Assign custom roles to Claude for specialized conversations
- **Model Selection**: Switch between different Claude models on the fly
- **Auto-Server Management**: Client automatically starts and manages server lifecycle

### Advanced Features

- **Session Persistence**: All conversations automatically saved to SQLite database
- **Smart Session Cleanup**: Unnamed sessions auto-prune, keeping only the most recent
- **Custom Terminal UI**: Rich formatting with colors, spinners, and progress indicators
- **Slash Commands**: 7 built-in commands for session and configuration management
- **HTTP REST API**: Full-featured server with Axum framework
- **Database Migrations**: Automatic schema management with SeaORM

## Architecture

### Client-Server Design

**Client (CLI)**

- Interactive terminal interface built with `console` and `indicatif`
- Session management with internal state tracking via `Cell<Option<i32>>`
- HTTP client for server communication using `reqwest`
- Command processor for slash command handling

**Server (HTTP API)**

- Axum web server on port 3500
- RESTful endpoints for sessions and chat
- SQLite database with SeaORM
- Claude API integration via Anthropic SDK
- Automatic database migrations on startup

### Workspace Structure

```
clai/
├── src/                        # CLI client
│   ├── main.rs                 # Application entry point
│   ├── commands.rs             # Slash command handlers
│   ├── sessions.rs             # Session management client
│   ├── input.rs                # Terminal input handling
│   └── utils/                  # CLI utilities
│
├── server/                     # HTTP server crate
│   ├── handlers/
│   │   ├── session.rs          # Session CRUD operations
│   │   ├── chat.rs             # Claude API integration
│   │   └── models.rs           # Model management
│   ├── db.rs                   # Database singleton
│   └── lib.rs                  # Server setup and routing
│
├── entity/                     # SeaORM entities
│   ├── sessions.rs             # Session entity model
│   └── messages.rs             # Message entity model
│
├── migration/                  # Database migrations
│   ├── m*_create_sessions.rs   # Sessions table
│   └── m*_create_messages.rs   # Messages table
│
├── common/                     # Shared code
│   ├── types.rs                # API request/response types
│   ├── error.rs                # Custom error types
│   ├── config.rs               # Environment configuration
│   └── constants.rs            # Application constants
│
└── mcp/                        # MCP client/server (WIP)
    ├── client.rs               # MCP client wrapper
    └── server.rs               # MCP server implementation
```

## What I Learned

### Rust Ecosystem & Patterns

**Async Runtime (Tokio)**

- Building async applications with `tokio`
- `#[tokio::main]` for async entry points
- Async trait methods and futures
- Proper async error handling
- Task spawning and concurrent operations

**SeaORM (Database)**

- Entity modeling with derive macros
- Active Record and Repository patterns
- Database migrations with `sea-orm-migration`
- Relationship mapping (one-to-many)
- Query building and execution
- Singleton database pattern with `OnceLock<Mutex<T>>`

**Axum (Web Framework)**

- HTTP routing and handlers
- Path and JSON extractors
- Response types and error handling
- Middleware and state management
- Integration with async ecosystem

**Error Handling**

- Custom error types with `thiserror`
- Domain-specific error variants
- Error propagation with `?` operator
- Converting between error types
- Macro-based error handling (`handle_db_operation!`)

**Workspace Management**

- Multi-crate Cargo workspace
- Shared dependencies in `[workspace.dependencies]`
- Internal crate dependencies
- Library vs binary organization
- Feature flags and optional dependencies

### Rust Language Features

**Interior Mutability**

- `Cell<Option<i32>>` for single-threaded mutable state
- `OnceLock<Mutex<T>>` for singleton pattern
- `RefCell` vs `Cell` trade-offs
- Thread-safe vs single-threaded mutability

**Smart Pointers & Ownership**

- `Arc` for shared ownership in async contexts
- `Mutex` for thread-safe interior mutability
- `MutexGuard` RAII pattern
- Lifetime management in complex types

**Traits & Generics**

- Trait bounds and where clauses
- Async traits with `async-trait`
- Generic functions and types
- Trait objects (`dyn Trait`)
- Derive macros (`Serialize`, `Deserialize`, etc.)

**Pattern Matching**

- Match expressions for command dispatch
- `if let` for optional values
- Pattern guards
- Destructuring in patterns

**Macros**

- Declarative macros (`macro_rules!`)
- Custom macros for repetitive code (`write_line!`, `write_spaced!`)
- Procedural derive macros (SeaORM)

### Third-Party Libraries

**Serde**

- JSON serialization/deserialization
- Custom serde attributes
- `#[serde(skip_serializing_if)]`
- Working with nested JSON structures

**Reqwest**

- HTTP client for API calls
- Request building and headers
- JSON request/response handling
- Error handling with HTTP statuses

**Clap**

- CLI argument parsing with derive API
- Command structure and subcommands
- Help text generation

**Console & Indicatif**

- Terminal styling and colors
- Progress bars and spinners
- Interactive prompts
- Terminal clearing and cursor control

**Chrono**

- Date/time handling
- `NaiveDateTime` for database timestamps
- Time formatting and parsing

### API Integration

**Anthropic Claude API**

- Chat completions endpoint
- Message history management
- System prompts and roles
- Token limits and max_tokens
- Model selection
- Error handling for API failures

### Database Design

**Schema Design**

- One-to-many relationships (sessions → messages)
- Timestamp tracking (created_at, updated_at)
- Optional fields for flexibility
- Auto-incrementing primary keys

**Migration Strategy**

- Sequential migration files
- Up/down migration support
- Automatic migration runner
- Schema versioning

### Development Practices

**Project Organization**

- Separation of concerns across crates
- Shared types in common crate
- Clear module boundaries
- Public vs private APIs

**Configuration Management**

- Environment variables with `envy`
- `.env` file support with `dotenv`
- Configuration validation
- Default values and fallbacks

**Testing Strategy**

- Integration tests in `tests/` directory
- Test utilities and fixtures
- Mocking HTTP clients
- Database testing patterns

**Error Messages**

- User-friendly error reporting
- Tracing for debugging
- Context-aware error messages

## Technical Highlights

- **~3,400 lines of Rust code** across 6 crates
- **Client-server architecture** with automatic lifecycle management
- **SQLite persistence** with SeaORM migrations
- **Rich terminal UI** with custom input handling
- **RESTful API** with Axum framework
- **Production-ready error handling** with custom error types
- **Workspace organization** with shared dependencies

## Getting Started

### Prerequisites

- Rust toolchain (nightly for `iter_map_windows` feature)
- Anthropic API key
- SQLite (handled automatically)

### Setup

```bash
# Clone and enter directory
cd ~/workspace/clai

# Create .env file
cat > .env << EOF
ANTHROPIC_API_KEY=sk-ant-...
DATABASE_URL=clai.db
CLAI_SERVER_URL=http://localhost:3500
EOF

# Run the application (auto-starts server)
cargo run
```

### Usage

**Chat with Claude:**

```
> Hello Claude!
Claude: Hello! How can I help you today?
```

**Slash Commands:**

- `/clear` - Clear the screen
- `/new [name]` - Start a new chat session
- `/save <name>` - Save current session with a name
- `/delete <name>` - Delete a saved session
- `/list` - List all saved sessions
- `/resume <name>` - Resume a saved session
- `/role [description]` - Set Claude's role
- `/model [model-id]` - Switch Claude model

**Exit:**

- Type `exit`, `quit`, or press `Ctrl+C`

## API Endpoints

The server exposes the following REST API:

- `POST /sessions` - Create new session
- `GET /sessions` - List all sessions
- `GET /sessions/:id` - Get session details
- `PUT /sessions/:id` - Update session
- `DELETE /sessions/:id` - Delete session
- `POST /sessions/:id/chat` - Send message
- `PUT /sessions/:id/role` - Set session role
- `PUT /sessions/:id/model` - Set session model
- `GET /models` - List available models

## Key Design Decisions

### Why Client-Server Architecture?

- **Persistence**: Server maintains database connection and state
- **API Key Security**: Claude API key never leaves server environment
- **Extensibility**: Multiple clients could connect to same server
- **Process Management**: Server can run independently or auto-start

### Why SeaORM?

- **Type Safety**: Compile-time query validation
- **Migrations**: Built-in migration system
- **Async**: Native async/await support
- **Active Record**: Clean, intuitive API

### Why Cell for Session State?

- **Single-threaded**: CLI is inherently single-threaded
- **Zero-cost**: No runtime overhead vs RefCell
- **Simple**: Clear ownership semantics

### Why Workspace?

- **Modularity**: Clear boundaries between components
- **Reusability**: Common code shared across crates
- **Compilation**: Parallel compilation of independent crates
- **Testing**: Isolated test suites per crate

## Future Enhancements

Potential areas for expansion:

- **MCP Integration**: Complete Model Context Protocol client/server for tool usage
  - Tool discovery and registration
  - `/mcp` command for tool invocation
  - Dynamic tool loading
- **Streaming Responses**: Real-time token streaming from Claude
- **Conversation History**: View and search past messages within CLI
- **Multi-user Support**: User authentication and isolation
- **Export Formats**: Export sessions to markdown, JSON, or plain text
- **Configuration UI**: Interactive configuration setup
- **Web Interface**: Browser-based client using same API
- **Claude Vision**: Image upload and analysis support

## Project Statistics

- **6 Cargo crates** in workspace
- **~3,400 lines** of Rust code
- **7 slash commands**
- **9 REST API endpoints**
- **2 database tables** with migrations
- **Async throughout** with Tokio runtime

## Key Takeaways

Building CLAI taught me that:

1. **Rust's Type System is Powerful**: Compile-time guarantees caught countless bugs before runtime
2. **Async Rust Works Well**: The tokio ecosystem is mature and well-integrated
3. **Error Handling Matters**: Custom error types with context make debugging dramatically easier
4. **Architecture Enables Growth**: Clear separation of concerns made adding features straightforward
5. **Workspace Organization Pays Off**: Multiple crates kept compilation fast and boundaries clear
6. **ORM Benefits Are Real**: SeaORM's type safety prevented SQL mistakes and simplified migrations
7. **CLI UX Is Important**: Rich terminal formatting and progress indicators significantly improve user experience
8. **Testing Strategies Evolve**: Different layers need different testing approaches (unit vs integration)

## Conclusion

CLAI demonstrates production-quality Rust development with modern async patterns, database integration, and API design. It serves as both a functional CLI application for daily use and a comprehensive reference for building client-server applications in Rust. The project showcases real-world patterns including async/await, database ORMs, HTTP frameworks, error handling, and workspace organization—all implemented independently as a learning exercise in advanced Rust development.
