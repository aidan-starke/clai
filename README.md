# CLAI - Command Line Artificial Interface

## Project Overview

CLAI is a Rust-based CLI chat application for interacting with Claude AI. It features a client-server architecture where the CLI client communicates with a local HTTP server that handles persistence and AI integration.

## Development Commands

```bash
cargo run                    # Run client (auto-starts server on port 3500)
cargo run -- --server        # Run server only

# Environment setup
# Create a .env file in the project root with:
# ANTHROPIC_API_KEY=sk-...  # Required for Claude AI integration
# DATABASE_URL=clai.db      # Optional (defaults to clai.db)
# CLAI_SERVER_URL=http://localhost:3500  # Optional server URL
```
