/// Writes a line of text to the terminal
#[macro_export]
macro_rules! write_line {
    ($($arg:tt)*) => {
        {
            let text = format!($($arg)*);
            if let Err(e) = crate::utils::get_term().write_line(&text) {
                eprintln!("Warning: Failed to write to terminal: {}", e);
            }
        }
    };
}

/// Writes a line of text with blank lines before and after for visual spacing
#[macro_export]
macro_rules! write_spaced {
    ($($arg:tt)*) => {
        {
            let text = format!($($arg)*);
            crate::utils::write_spaced_line(&text);
        }
    };
}
