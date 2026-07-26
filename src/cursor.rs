const CURSOR_BLOCK: usize = 2;
const CURSOR_UNDERLINE: usize = 4;
const CURSOR_BAR: usize = 6;

use std::cmp::{max, min};

use crate::{
    app::Mode,
    document::{Document, Pos},
    viewport::Viewport,
};

#[derive(Copy, Clone)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { row: 0, col: 0 }
    }

    pub fn build(&self, doc: &Document, viewport: &Viewport, mode: Mode) -> String {
        use Mode::*;

        let mut building = String::new();

        let col = min(self.col, doc.col_bound(self.row, mode));

        building.push_str(&format!(
            "\x1b[{};{}H",
            self.row - viewport.top_row + 1,
            col - viewport.left_column + 1
        ));

        let mode = match mode {
            Normal | Visual(_) => CURSOR_BLOCK,
            Replace | Delete => CURSOR_UNDERLINE,
            Insert => CURSOR_BAR,
        };

        building.push_str(&format!("\x1b[{} q", mode));

        building
    }

    pub fn bound_col(&mut self, doc: &Document, mode: Mode) -> &mut Self {
        self.col = max(min(self.col, doc.col_bound(self.row, mode)), 0);

        self
    }

    pub fn bound_row(&mut self, doc: &Document) -> &mut Self {
        self.row = max(min(self.row, doc.row_bound()), 0);

        self
    }

    pub fn left(&mut self) -> &mut Self {
        if self.col > 0 {
            self.col -= 1;
        }

        self
    }

    pub fn right(&mut self, doc: &Document, mode: Mode) -> &mut Self {
        self.col += 1;
        self.bound_col(doc, mode);

        self
    }

    pub fn down(&mut self, doc: &Document) -> &mut Self {
        self.row += 1;
        self.bound_row(doc);

        self
    }

    pub fn up(&mut self) -> &mut Self {
        if self.row > 0 {
            self.row -= 1;
        }

        self
    }

    pub fn go_to_first_line(&mut self) -> &mut Self {
        self.row = 0;

        self
    }

    pub fn go_to_last_char(&mut self, doc: &Document) -> &mut Self {
        self.row = doc.row_bound();
        self.col = doc.col_bound(self.row, Mode::Normal);

        self
    }

    pub fn go_to_start_of_line(&mut self, doc: &Document) -> &mut Self {
        self.col = doc.get_content()[self.row]
            .chars()
            .enumerate()
            .find(|(_, ch)| *ch != ' ')
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        self
    }

    pub fn go_to_end_of_line(&mut self, doc: &Document, mode: Mode) -> &mut Self {
        self.col = doc.col_bound(self.row, mode);

        self
    }

    pub fn to_pos(&self) -> Pos {
        Pos {
            row: self.row,
            col: self.col,
        }
    }

    pub fn go_to_pos(&mut self, pos: Pos) -> &mut Self {
        self.row = pos.row;
        self.col = pos.col;

        self
    }
}
//     pub fn go_to_next_word(&mut self, screen: &ScreenBuffer) {
//         let mut idx = self.y * screen.width + self.x;
//
//         enum State {
//             Alphabetic,
//             NonAlphabetic,
//             WhiteSpace,
//         }
//
//         let state = match screen.cells[idx].char {
//             c if c.is_alphanumeric() => State::Alphabetic,
//             c if is_whitespace(c) => State::WhiteSpace,
//             _ => State::NonAlphabetic,
//         };
//
//         for (i, c) in screen.cells.iter().skip(idx).enumerate() {
//             match (&state, c.char) {
//                 (State::Alphabetic, c) if !c.is_alphanumeric() => {
//                     idx += i;
//                     break;
//                 }
//                 (State::NonAlphabetic, c) if c.is_alphanumeric() || is_whitespace(c) => {
//                     idx += i;
//                     break;
//                 }
//                 (State::WhiteSpace, c) if !is_whitespace(c) => {
//                     idx += i;
//                     break;
//                 }
//                 _ => {}
//             }
//         }
//
//         for (i, c) in screen.cells.iter().skip(idx).enumerate() {
//             match c.char {
//                 c if !is_whitespace(c) => {
//                     idx += i;
//                     break;
//                 }
//                 _ => {}
//             }
//         }
//
//         self.x = idx % screen.width;
//         self.y = idx / screen.width;
//     }
//
//     pub fn go_to_prev_word(&mut self, screen: &ScreenBuffer) {
//         let idx = self.y * screen.width + self.x;
//
//         if idx == 0 {
//             return;
//         }
//
//         enum State {
//             Alphabetic,
//             NonAlphabetic,
//             WhiteSpace,
//         }
//
//         let mut state = match screen.cells[idx - 1].char {
//             c if c.is_alphanumeric() => State::Alphabetic,
//             c if is_whitespace(c) => State::WhiteSpace,
//             _ => State::NonAlphabetic,
//         };
//
//         let mut step = idx;
//
//         for (i, c) in screen.cells.iter().take(idx - 1).rev().enumerate() {
//             match (&state, c.char) {
//                 (State::Alphabetic, c) if !c.is_alphanumeric() => {
//                     step = i + 1;
//                     break;
//                 }
//                 (State::NonAlphabetic, c) if c.is_alphanumeric() || is_whitespace(c) => {
//                     step = i + 1;
//                     break;
//                 }
//                 (State::WhiteSpace, c) if !is_whitespace(c) => {
//                     if c.is_alphanumeric() {
//                         state = State::Alphabetic;
//                     } else {
//                         state = State::NonAlphabetic;
//                     }
//                 }
//                 _ => {}
//             }
//         }
//
//         self.x = (idx - step) % screen.width;
//         self.y = (idx - step) / screen.width;
//     }
//
//     pub fn go_to_last_char_of_next_word(&mut self, screen: &ScreenBuffer) {
//         let idx = self.y * screen.width + self.x;
//
//         enum State {
//             Alphabetic,
//             NonAlphabetic,
//             WhiteSpace,
//         }
//
//         let mut state = match screen.cells[idx + 1].char {
//             c if c.is_alphanumeric() => State::Alphabetic,
//             c if is_whitespace(c) => State::WhiteSpace,
//             _ => State::NonAlphabetic,
//         };
//
//         let mut step = 0;
//
//         let eof = (screen.line_count - 1) * screen.width + screen.line_len(screen.line_count - 1);
//
//         for (i, c) in screen.cells.iter().skip(idx + 1).enumerate() {
//             match (&state, c.char) {
//                 (State::Alphabetic, c) if !c.is_alphanumeric() => {
//                     step = i;
//                     break;
//                 }
//                 (State::NonAlphabetic, c) if c.is_alphanumeric() || is_whitespace(c) => {
//                     step = i;
//                     break;
//                 }
//                 (State::WhiteSpace, c) if !is_whitespace(c) => {
//                     if c.is_alphanumeric() {
//                         state = State::Alphabetic;
//                     } else {
//                         state = State::NonAlphabetic;
//                     }
//                 }
//                 _ if i >= eof => break,
//                 _ => {}
//             }
//         }
//
//         self.x = (idx + step) % screen.width;
//         self.y = (idx + step) / screen.width;
//     }
// }
