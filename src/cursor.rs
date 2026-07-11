use std::cmp::{max, min};

use crate::screen::{Mode, ScreenBuffer};

pub const CURSOR_BLOCK: usize = 2;
const CURSOR_UNDERLINE: usize = 4;
const CURSOR_BAR: usize = 6;

#[derive(Copy, Clone)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { x: 0, y: 0 }
    }

    pub fn build(&self, screen: &ScreenBuffer, mode: Mode) -> String {
        let mut building = String::new();

        building.push_str(&format!(
            "\x1b[{};{}H",
            self.y + 1,
            x_bounded(self.x as isize, self.y, screen, mode) + 1
        ));

        let mode = match mode {
            Mode::Normal => CURSOR_BLOCK,
            Mode::Undo => CURSOR_BLOCK,
            Mode::Replace => CURSOR_UNDERLINE,
            Mode::Delete => CURSOR_UNDERLINE,
            Mode::Insert => CURSOR_BAR,
        };

        building.push_str(&format!("\x1b[{} q", mode));

        building
    }

    pub fn reset(&mut self, screen: &ScreenBuffer, mode: Mode) {
        self.x = x_bounded(self.x as isize, self.y, screen, mode)
    }

    pub fn left(&mut self, screen: &ScreenBuffer, mode: Mode) {
        self.x = x_bounded(self.x as isize - 1, self.y, screen, mode);
    }

    pub fn right(&mut self, screen: &ScreenBuffer, mode: Mode) {
        self.x = x_bounded(self.x as isize + 1, self.y, screen, mode);
    }

    pub fn down(&mut self, screen: &ScreenBuffer) {
        self.y = y_bounded(self.y as isize + 1, screen);
    }

    pub fn up(&mut self, screen: &ScreenBuffer) {
        self.y = y_bounded(self.y as isize - 1, screen);
    }

    pub fn go_to_line_start(&mut self, screen: &ScreenBuffer) {
        let start = self.y * screen.width;
        let end = start + screen.line_len(self.y);

        self.x = 0;

        for i in start..end {
            self.x = i % screen.width;

            if screen.cells[i].char != ' ' && screen.cells[i].char != '·' {
                break;
            }
        }
    }

    pub fn go_to_line_end(&mut self, screen: &ScreenBuffer, mode: Mode) {
        self.x = x_bounded(screen.width as isize, self.y, screen, mode);
    }

    pub fn go_to_next_word(&mut self, screen: &ScreenBuffer) {
        let mut idx = self.y * screen.width + self.x;

        enum State {
            Alphabetic,
            NonAlphabetic,
            WhiteSpace,
        }

        let state = match screen.cells[idx].char {
            c if c.is_alphanumeric() => State::Alphabetic,
            c if is_whitespace(c) => State::WhiteSpace,
            _ => State::NonAlphabetic,
        };

        for (i, c) in screen.cells.iter().skip(idx).enumerate() {
            match (&state, c.char) {
                (State::Alphabetic, c) if !c.is_alphanumeric() => {
                    idx += i;
                    break;
                }
                (State::NonAlphabetic, c) if c.is_alphanumeric() || is_whitespace(c) => {
                    idx += i;
                    break;
                }
                (State::WhiteSpace, c) if !is_whitespace(c) => {
                    idx += i;
                    break;
                }
                _ => {}
            }
        }

        for (i, c) in screen.cells.iter().skip(idx).enumerate() {
            match c.char {
                c if !is_whitespace(c) => {
                    idx += i;
                    break;
                }
                _ => {}
            }
        }

        self.x = idx % screen.width;
        self.y = idx / screen.width;
    }

    pub fn go_to_prev_word(&mut self, screen: &ScreenBuffer) {
        let mut idx = self.y * screen.width + self.x;

        enum State {
            Alphabetic,
            NonAlphabetic,
            WhiteSpace,
        }

        let mut state = match screen.cells[idx - 1].char {
            c if c.is_alphanumeric() => State::Alphabetic,
            c if is_whitespace(c) => State::WhiteSpace,
            _ => State::NonAlphabetic,
        };

        for (i, c) in screen.cells.iter().take(idx - 1).rev().enumerate() {
            match (&state, c.char) {
                (State::Alphabetic, c) if !c.is_alphanumeric() => {
                    idx -= i + 1;
                    break;
                }
                (State::NonAlphabetic, c) if c.is_alphanumeric() || is_whitespace(c) => {
                    idx -= i + 1;
                    break;
                }
                (State::WhiteSpace, c) if !is_whitespace(c) => {
                    if c.is_alphanumeric() {
                        state = State::Alphabetic;
                    } else {
                        state = State::NonAlphabetic;
                    }
                }
                _ => {}
            }
        }

        self.x = idx % screen.width;
        self.y = idx / screen.width;
    }

    pub fn go_to_last_char_of_next_word(&mut self, screen: &ScreenBuffer) {
        let mut idx = self.y * screen.width + self.x;

        enum State {
            Alphabetic,
            NonAlphabetic,
            WhiteSpace,
        }

        let mut state = match screen.cells[idx + 1].char {
            c if c.is_alphanumeric() => State::Alphabetic,
            c if is_whitespace(c) => State::WhiteSpace,
            _ => State::NonAlphabetic,
        };

        for (i, c) in screen.cells.iter().skip(idx + 1).enumerate() {
            match (&state, c.char) {
                (State::Alphabetic, c) if !c.is_alphanumeric() => {
                    idx += i;
                    break;
                }
                (State::NonAlphabetic, c) if c.is_alphanumeric() || is_whitespace(c) => {
                    idx += i;
                    break;
                }
                (State::WhiteSpace, c) if !is_whitespace(c) => {
                    if c.is_alphanumeric() {
                        state = State::Alphabetic;
                    } else {
                        state = State::NonAlphabetic;
                    }
                }
                _ => {}
            }
        }

        self.x = idx % screen.width;
        self.y = idx / screen.width;
    }
}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '·'
}

pub fn x_bounded(x: isize, y: usize, screen: &ScreenBuffer, mode: Mode) -> usize {
    let base_bound = max(screen.line_len(y) as isize - 1, 0) as usize;

    let bound = match (mode, screen.line_len(y)) {
        (Mode::Insert, 0) => 0,
        (Mode::Insert, _) => base_bound + 1,
        _ => base_bound,
    };

    max(min(bound as isize, x), 0) as usize
}

pub fn y_bounded(y: isize, screen: &ScreenBuffer) -> usize {
    max(min(screen.line_count as isize - 1, y), 0) as usize
}
