const CURSOR_BLOCK: usize = 2;
const CURSOR_UNDERLINE: usize = 4;
const CURSOR_BAR: usize = 6;

use std::cmp::{max, min};

use crate::{app::Mode, document::Document, viewport::Viewport};

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
        let mut building = String::new();

        let col = min(self.col, Cursor::col_bound(self.row, doc, mode));

        building.push_str(&format!(
            "\x1b[{};{}H",
            self.row + viewport.top_row + 1,
            col + viewport.left_column + 1
        ));

        let mode = match mode {
            Mode::Normal => CURSOR_BLOCK,
            Mode::Visual(_) => CURSOR_BLOCK,
            Mode::Undo => CURSOR_BLOCK,
            Mode::Replace => CURSOR_UNDERLINE,
            Mode::Delete => CURSOR_UNDERLINE,
            Mode::Insert => CURSOR_BAR,
        };

        building.push_str(&format!("\x1b[{} q", mode));

        building
    }

    fn col_bound(row: usize, doc: &Document, mode: Mode) -> usize {
        match mode {
            Mode::Insert => doc.get_content()[row].len(),
            _ => max(doc.get_content()[row].len() as isize - 1, 0) as usize,
        }
    }

    fn row_bound(doc: &Document) -> usize {
        max(doc.get_content().len() as isize - 2, 0) as usize
    }

    fn bound(col: isize, row: isize, doc: &Document, mode: Mode) -> (usize, usize) {
        let (col, row) = (max(col, 0) as usize, max(row, 0) as usize);

        (
            max(min(col, Cursor::col_bound(row, doc, mode)), 0),
            max(min(row, Cursor::row_bound(doc)), 0),
        )
    }

    pub fn left(&mut self, doc: &Document, mode: Mode) {
        (self.col, _) = Cursor::bound(self.col as isize - 1, self.row as isize, doc, mode);
    }

    pub fn right(&mut self, doc: &Document, mode: Mode) {
        (self.col, _) = Cursor::bound(self.col as isize + 1, self.row as isize, doc, mode);
    }

    pub fn down(&mut self, doc: &Document, mode: Mode) {
        (_, self.row) = Cursor::bound(self.col as isize, self.row as isize + 1, doc, mode);
    }

    pub fn up(&mut self, doc: &Document, mode: Mode) {
        (_, self.row) = Cursor::bound(self.col as isize, self.row as isize - 1, doc, mode);
    }
}

//     pub fn go_to_line_start(&mut self, screen: &ScreenBuffer) {
//         let start = self.y * screen.width;
//         let end = start + screen.line_len(self.y);
//
//         self.x = 0;
//
//         for i in start..end {
//             self.x = i % screen.width;
//
//             if screen.cells[i].char != ' ' && screen.cells[i].char != '·' {
//                 break;
//             }
//         }
//     }
//
//     pub fn go_to_line_end(&mut self, screen: &ScreenBuffer, mode: Mode) {
//         self.x = x_bounded(screen.width as isize, self.y, screen, mode);
//     }
//
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
//
// fn is_whitespace(c: char) -> bool {
//     c == ' ' || c == '·'
// }
//
// pub fn x_bounded(x: isize, y: usize, screen: &ScreenBuffer, mode: Mode) -> usize {
//     let base_bound = max(screen.line_len(y) as isize - 1, 0) as usize;
//
//     let bound = match (mode, screen.line_len(y)) {
//         (Mode::Insert, 0) => 0,
//         (Mode::Insert, _) => base_bound + 1,
//         _ => base_bound,
//     };
//
//     max(min(bound as isize, x), 0) as usize
// }
//
// pub fn y_bounded(y: isize, screen: &ScreenBuffer) -> usize {
//     max(min(screen.line_count as isize - 1, y), 0) as usize
// }
