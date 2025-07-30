use crate::utils::{constants::COMMANDS, get_term};
use console::{style, Key, Term};

pub struct InputReader {
    term: &'static Term,
    input: String,
    show_dropdown: bool,
    selected_index: usize,
    dropdown_lines: usize,
}

impl InputReader {
    pub fn new() -> Self {
        Self {
            term: get_term(),
            input: String::new(),
            show_dropdown: false,
            selected_index: 0,
            dropdown_lines: 0,
        }
    }

    pub fn read_line(&mut self) -> std::result::Result<String, std::io::Error> {
        self.input.clear();
        self.show_dropdown = false;
        self.selected_index = 0;
        self.dropdown_lines = 0;

        loop {
            // Clear any existing dropdown
            self.clear_dropdown()?;

            // Show current input
            self.render()?;

            // Show dropdown if needed
            if self.show_dropdown {
                self.update_dropdown()?;
            }

            // Read next key
            let key = self.term.read_key()?;

            match self.handle_key(key)? {
                Some(result) => {
                    self.finalize_input(&result)?;
                    return Ok(result);
                }
                None => continue,
            }
        }
    }

    fn handle_key(&mut self, key: Key) -> std::result::Result<Option<String>, std::io::Error> {
        match key {
            Key::Enter => {
                // Select command if dropdown is showing
                if self.show_dropdown {
                    let filtered_commands = Self::get_filtered_commands(&self.input);
                    if !filtered_commands.is_empty()
                        && self.selected_index < filtered_commands.len()
                    {
                        self.input = filtered_commands[self.selected_index].to_string();
                    }
                }
                Ok(Some(self.input.clone()))
            }
            Key::Escape => {
                self.show_dropdown = false;
                self.selected_index = 0;
                Ok(None)
            }
            Key::ArrowDown => {
                if self.show_dropdown {
                    let filtered_commands = Self::get_filtered_commands(&self.input);
                    if !filtered_commands.is_empty() {
                        self.selected_index = (self.selected_index + 1) % filtered_commands.len();
                    }
                }
                Ok(None)
            }
            Key::ArrowUp => {
                if self.show_dropdown {
                    let filtered_commands = Self::get_filtered_commands(&self.input);
                    if !filtered_commands.is_empty() {
                        self.selected_index = if self.selected_index == 0 {
                            filtered_commands.len() - 1
                        } else {
                            self.selected_index - 1
                        };
                    }
                }
                Ok(None)
            }
            Key::Backspace => {
                if !self.input.is_empty() {
                    self.input.pop();
                    if !self.input.starts_with('/') {
                        self.show_dropdown = false;
                        self.selected_index = 0;
                    }
                }
                Ok(None)
            }
            Key::Char(c) => {
                self.input.push(c);
                if self.input.starts_with('/') {
                    self.show_dropdown = true;
                    self.selected_index = 0;
                } else {
                    self.show_dropdown = false;
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn clear_dropdown(&mut self) -> std::result::Result<(), std::io::Error> {
        if self.dropdown_lines > 0 {
            for _ in 0..self.dropdown_lines {
                self.term.move_cursor_down(1)?;
                self.term.clear_line()?;
            }
            for _ in 0..self.dropdown_lines {
                self.term.move_cursor_up(1)?;
            }
            self.dropdown_lines = 0;
        }
        Ok(())
    }

    fn render(&self) -> std::result::Result<(), std::io::Error> {
        self.term.clear_line()?;
        if self.input.is_empty() {
            self.term.write_line("")?;
        }
        self.term
            .write_str(&format!("{}: {}", style("You").green(), self.input))?;
        Ok(())
    }

    fn update_dropdown(&mut self) -> std::result::Result<(), std::io::Error> {
        let filtered_commands = Self::get_filtered_commands(&self.input);

        if !filtered_commands.is_empty() {
            let num_lines = filtered_commands.len();
            self.dropdown_lines = num_lines;
            self.term.write_line("")?;
            for (i, cmd) in filtered_commands.iter().enumerate() {
                if i == self.selected_index {
                    self.term
                        .write_line(&format!("  → {}", style(cmd).bold()))?;
                } else {
                    self.term.write_line(&format!("    {}", style(cmd).dim()))?;
                }
            }
            // Move cursor back to input line
            for _ in 0..num_lines {
                self.term.move_cursor_up(1)?;
            }
            self.term.move_cursor_up(1)?; // Go above the empty line too
        }
        Ok(())
    }

    fn finalize_input(&mut self, result: &str) -> std::result::Result<(), std::io::Error> {
        if self.dropdown_lines > 0 {
            for _ in 0..self.dropdown_lines {
                self.term.move_cursor_down(1)?;
                self.term.clear_line()?;
            }
            for _ in 0..self.dropdown_lines {
                self.term.move_cursor_up(1)?;
            }
            self.term.move_cursor_down(1)?; // Move past the empty line
            self.term.clear_line()?;
            self.term.move_cursor_up(1)?;
        }

        self.term.clear_line()?;
        self.term.write_str(&format!("You: {}", result))?;
        self.term.write_line("")?;
        Ok(())
    }

    fn get_filtered_commands(input: &str) -> Vec<&str> {
        COMMANDS
            .iter()
            .filter(|cmd| cmd.starts_with(input))
            .copied()
            .collect()
    }
}

impl Default for InputReader {
    fn default() -> Self {
        Self::new()
    }
}

