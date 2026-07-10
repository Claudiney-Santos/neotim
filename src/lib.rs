pub mod bindings;
pub mod cursor;
pub mod screen;

pub const BREAK_LINE: &str = "\r\n";

pub const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H"; // Clears entire screen, moves cursor to top-left
pub const CLEAR_LINE: &str = "\x1b[2K"; // Clears the entire current line
pub const CLEAR_TO_END: &str = "\x1b[0K"; // Clears from cursor to end of line
pub const CLEAR_TO_START: &str = "\x1b[1K"; // Clears from cursor to start of line

pub const CURSOR_UP: &str = "\x1b[A"; // Moves cursor up one cell
pub const CURSOR_DOWN: &str = "\x1b[B"; // Moves cursor down one cell
pub const CURSOR_RIGHT: &str = "\x1b[C"; // Moves cursor right one cell
pub const CURSOR_LEFT: &str = "\x1b[D"; // Moves cursor left one cell
pub const CURSOR_HOME: &str = "\x1b[H"; // Moves cursor to top-left (0,0)
pub const SAVE_CURSOR: &str = "\x1b[s"; // Saves current cursor position
pub const RESTORE_CURSOR: &str = "\x1b[u"; // Restores saved cursor position
pub const HIDE_CURSOR: &str = "\x1b[?25l"; // Hides the cursor completely
pub const SHOW_CURSOR: &str = "\x1b[?25h"; // Shows the cursor

pub const RESET_ALL: &str = "\x1b[0m"; // Resets all colors and styles
pub const STYLE_BOLD: &str = "\x1b[1m"; // Bold text
pub const STYLE_DIM: &str = "\x1b[2m"; // Dim/faint text
pub const STYLE_ITALIC: &str = "\x1b[3m"; // Italic text
pub const STYLE_UNDERLINE: &str = "\x1b[4m"; // Underlined text
pub const STYLE_BLINK: &str = "\x1b[5m"; // Blinking text
pub const STYLE_INVERT: &str = "\x1b[7m"; // Swaps foreground and background colors
pub const STYLE_HIDDEN: &str = "\x1b[8m"; // Invisible text
pub const STYLE_STRIKE: &str = "\x1b[9m"; // Strikethrough text

pub const FG_BLACK: &str = "\x1b[30m";
pub const FG_RED: &str = "\x1b[31m";
pub const FG_GREEN: &str = "\x1b[32m";
pub const FG_YELLOW: &str = "\x1b[33m";
pub const FG_BLUE: &str = "\x1b[34m";
pub const FG_MAGENTA: &str = "\x1b[35m";
pub const FG_CYAN: &str = "\x1b[36m";
pub const FG_WHITE: &str = "\x1b[37m";
pub const FG_DEFAULT: &str = "\x1b[39m";

pub const BG_RED: &str = "\x1b[41m";
pub const BG_GREEN: &str = "\x1b[42m";
pub const BG_YELLOW: &str = "\x1b[43m";
pub const BG_BLUE: &str = "\x1b[44m";
pub const BG_MAGENTA: &str = "\x1b[45m";
pub const BG_CYAN: &str = "\x1b[46m";
pub const BG_WHITE: &str = "\x1b[47m";
pub const BG_DEFAULT: &str = "\x1b[49m";

#[macro_export]
macro_rules! prin {
    ($($arg:tt)*) => {{
        print!($($arg)*);
        std::io::Write::flush(&mut std::io::stdout()).expect("Error on flush prin!");
    }};
}
