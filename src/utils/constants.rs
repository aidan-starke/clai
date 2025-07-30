pub const COMMANDS: [&str; 8] = [
    "/clear", "/new", "/save", "/delete", "/list", "/resume", "/role", "/model",
];

pub const DEFAULT_SERVER_HOST: &str = "0.0.0.0";
pub const DEFAULT_SERVER_PORT: u16 = 3500;
pub const DEFAULT_SERVER_URL: &str = "http://localhost:3500";

pub const CLAUDE_MAX_TOKENS: i32 = 1000;

pub const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";