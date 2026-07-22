pub mod context;
pub mod cursor;
pub mod document;
pub mod error;
pub mod render_buffer;
pub mod terminal;
pub mod undo;
pub mod viewport;

pub const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";
pub const HIDE_CURSOR: &str = "\x1b[?25l";
pub const SHOW_CURSOR: &str = "\x1b[?25h";
pub const BACKSPACE: char = '\x7F';
pub const ENTER: char = '\r';
pub const ESC: char = '\x1b';

#[macro_export]
macro_rules! prin {
    ($($arg:tt)*) => {{
        print!($($arg)*);
        std::io::Write::flush(&mut std::io::stdout()).expect("Error on flush prin!");
    }};
}
