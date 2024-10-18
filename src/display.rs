use crossterm::{
    cursor::MoveTo,
    execute,
    style::Print,
    terminal::{size, Clear, ClearType},
};
use std::io::{stdout, Write};

pub fn display_message(message: String) {
    // Get terminal size
    let (term_width, term_height) = size().unwrap();

    // Calculate horizontal and vertical centering
    let message_lines = message.lines().count();
    let message_length = message.lines().map(|line| line.len()).max().unwrap_or(0);
    let x_pos = (term_width / 2).saturating_sub((message_length / 2) as u16);
    let y_pos = (term_height / 2).saturating_sub((message_lines / 2) as u16);

    // Clear the terminal and move the cursor to the center
    execute!(stdout(), Clear(ClearType::All)).unwrap();
    execute!(stdout(), MoveTo(x_pos, y_pos)).unwrap();

    // Print the message at the calculated position
    execute!(stdout(), Print(&message)).unwrap();

    stdout().flush().unwrap();
}
