use std::{
    cmp::min,
    io::{self, Read, stdin},
};

use crate::{
    BACKSPACE, ENTER, ESC,
    cursor::Cursor,
    document::{Document, Pos},
    undo::UndoStack,
    viewport::Viewport,
};

#[derive(Clone)]
pub enum Clipboard {
    Normal(String),
    Line(String),
    None,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    Replace,
    Delete,
    Insert,
    Visual(Pos),
}

impl Mode {
    pub fn set(&mut self, mode: Mode) -> Mode {
        *self = mode;
        mode
    }
}

pub struct App {
    pub doc: Document,
    pub viewport: Viewport,
    pub cursor: Cursor,
    pub mode: Mode,
    pub undo: UndoStack,
    pub clipboard: Clipboard,
}

pub fn get_key_pressed() -> io::Result<char> {
    let mut key = [0; 1];
    stdin().read_exact(&mut key)?;

    Ok(key[0] as char)
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            doc: Document::new()?,
            viewport: Viewport::new(),
            cursor: Cursor::new(),
            mode: Mode::Normal,
            undo: UndoStack::new(),
            clipboard: Clipboard::None,
        })
    }

    pub fn handle_input(&mut self, key: char) -> anyhow::Result<bool> {
        use Mode::*;

        let App {
            cursor,
            doc,
            mode,
            undo,
            clipboard,
            ..
        } = self;

        match (*mode, key) {
            (Normal | Visual(_), 'Q') => return Ok(false),
            (Normal | Visual(_), 'W') => doc.save()?,
            (Normal | Visual(_), 'h') => {
                cursor.bound_col(doc, *mode).left();
            }
            (Normal | Visual(_), 'j') => {
                cursor.down(doc);
            }
            (Normal | Visual(_), 'k') => {
                cursor.up();
            }
            (Normal | Visual(_), 'l') => {
                cursor.right(doc, *mode);
            }
            (Normal, 'i') => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                cursor.bound_col(doc, mode.set(Insert));
            }
            (Normal | Visual(_), 'u') => {
                if let Some(snapshot) = undo.pop() {
                    doc.restore(snapshot.0);
                    *cursor = snapshot.1;
                    mode.set(snapshot.2);
                }
            }
            (Normal, 'I') => {
                mode.set(Insert);
                cursor.go_to_start_of_line(doc);
            }
            (Normal | Visual(_), 'w') => {
                cursor.go_to_pos(doc.next_word(cursor.to_pos()));
            }
            (Normal | Visual(_), 'b') => {
                cursor.go_to_pos(doc.prev_word(cursor.to_pos()));
            }
            (Normal | Visual(_), 'e') => {
                cursor.go_to_pos(doc.last_char_of_next_word(cursor.to_pos()));
            }
            (Normal, 'A') => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                cursor.go_to_end_of_line(doc, mode.set(Insert));
            }
            (Normal, 's') => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                mode.set(Insert);
                doc.delete(cursor.to_pos(), cursor.to_pos());
            }
            (Normal, 'a') => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                mode.set(Insert);
                cursor.right(doc, self.mode);
            }
            (Normal, 'o') => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                mode.set(Insert);
                doc.insert_line(cursor.row + 1);
                cursor.down(doc);
            }
            (Replace, ch) => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                if !ch.is_control() {
                    doc.delete(cursor.to_pos(), cursor.to_pos());
                    doc.insert(cursor.to_pos(), &ch.to_string());
                }

                mode.set(Normal);
            }
            (Normal, 'r') => {
                mode.set(Replace);
            }
            (Normal, 'x') => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                *clipboard = Clipboard::Normal(doc.delete(cursor.to_pos(), cursor.to_pos()));
                cursor.bound_col(doc, *mode);
            }

            (Insert, ESC) => {
                cursor.bound_col(doc, mode.set(Normal));
            }
            (Normal, 'v') => {
                mode.set(Visual(cursor.to_pos()));
            }
            (Visual(_), ESC) => {
                mode.set(Normal);
            }
            (Visual(landmark), 'y') => {
                let cursor_pos = cursor.clone().bound_col(doc, *mode).to_pos();
                *clipboard = Clipboard::Normal(doc.copy(landmark, cursor_pos));
                cursor
                    .go_to_pos(min(landmark, cursor_pos))
                    .bound_row(doc)
                    .bound_col(doc, *mode);
                mode.set(Normal);
            }
            (Visual(landmark), 'Y') => {
                let cursor_pos = cursor.clone().bound_col(doc, *mode).to_pos();

                let (mut start, mut end) = if landmark < cursor_pos {
                    (landmark, cursor_pos)
                } else {
                    (cursor_pos, landmark)
                };

                start.col = 0;
                end.col = doc.col_bound(end.row, *mode);

                *clipboard = Clipboard::Line(doc.copy(start, end));
                cursor.go_to_pos(start).bound_row(doc).bound_col(doc, *mode);
                mode.set(Normal);
            }
            (Visual(landmark), 'D') => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                let cursor_pos = cursor.clone().bound_col(doc, *mode).to_pos();

                let (mut start, mut end) = if landmark < cursor_pos {
                    (landmark, cursor_pos)
                } else {
                    (cursor_pos, landmark)
                };

                start.col = 0;
                end.col = doc.col_bound(end.row, *mode);

                *clipboard = Clipboard::Line(doc.delete(start, end));
                cursor.go_to_pos(start).bound_row(doc).bound_col(doc, *mode);
                mode.set(Normal);
            }
            (Visual(landmark), 'd') => {
                undo.push(doc.snapshot(), cursor.clone(), *mode);
                let cursor_pos = cursor.clone().bound_col(doc, *mode).to_pos();

                *clipboard = Clipboard::Normal(doc.delete(landmark, cursor_pos));
                cursor
                    .go_to_pos(min(landmark, cursor_pos))
                    .bound_row(doc)
                    .bound_col(doc, *mode);
                mode.set(Normal);
            }
            (Normal, 'P') => {
                undo.push(doc.snapshot(), cursor.clone(), Normal);

                match clipboard {
                    Clipboard::Normal(s) => {
                        cursor.go_to_pos(doc.insert(cursor.clone().to_pos(), &s));
                    }
                    Clipboard::Line(s) => {
                        doc.insert(
                            Pos {
                                row: cursor.row,
                                col: 0,
                            },
                            &s,
                        );
                    }
                    _ => {}
                }
            }
            (Normal, 'p') => {
                undo.push(doc.snapshot(), cursor.clone(), Normal);
                match clipboard {
                    Clipboard::Normal(s) => {
                        let end_pos = doc.insert(cursor.clone().right(doc, Insert).to_pos(), &s);
                        cursor.go_to_pos(end_pos);
                    }
                    Clipboard::Line(s) => {
                        if s.ends_with("\n") {
                            s.pop();
                        }

                        doc.insert(
                            cursor.clone().go_to_end_of_line(doc, Insert).to_pos(),
                            &format!("\n{s}"),
                        );
                        cursor.down(doc);
                    }
                    _ => {}
                }
            }
            (Delete, 'd') => {
                undo.push(doc.snapshot(), cursor.clone(), Normal);
                let start = Pos {
                    row: cursor.row,
                    col: 0,
                };

                let end = cursor.clone().go_to_end_of_line(doc, Insert).to_pos();

                *clipboard = Clipboard::Line(doc.delete(start, end));
                cursor.bound_row(doc);
                mode.set(Normal);
            }
            (Delete, 'j') => {
                undo.push(doc.snapshot(), cursor.clone(), Normal);
                if doc.row_bound() <= cursor.row {
                    mode.set(Normal);
                    return Ok(true);
                }

                let start = Pos {
                    row: cursor.row,
                    col: 0,
                };
                let end = cursor
                    .clone()
                    .down(doc)
                    .go_to_end_of_line(doc, Insert)
                    .to_pos();

                *clipboard = Clipboard::Line(doc.delete(start, end));
                cursor.bound_row(doc);
                mode.set(Normal);
            }
            (Delete, 'k') => {
                undo.push(doc.snapshot(), cursor.clone(), Normal);
                if cursor.row == 0 {
                    mode.set(Normal);
                    return Ok(true);
                }

                let start = Pos {
                    row: cursor.row - 1,
                    col: 0,
                };
                let end = cursor.clone().go_to_end_of_line(doc, Insert).to_pos();

                *clipboard = Clipboard::Line(doc.delete(start, end));
                cursor.up().bound_row(doc);
                mode.set(Normal);
            }
            (Delete, _) => {
                mode.set(Normal);
            }
            (Normal, 'd') => {
                mode.set(Delete);
            }
            (Normal, 'g') => {
                let key = get_key_pressed()?;

                if key != 'g' {
                    return self.handle_input(key);
                }

                cursor.go_to_first_line();
            }
            (Normal | Visual(_), 'G') => {
                cursor.go_to_last_char(doc);
            }
            (Normal, ENTER) => {
                cursor.down(doc).go_to_start_of_line(doc);
            }
            (Normal, BACKSPACE) => {
                if cursor.col == 0 {
                    cursor.up().go_to_end_of_line(doc, *mode);
                    return Ok(true);
                }

                cursor.left();
            }
            (Insert, ENTER) => {
                doc.insert(cursor.to_pos(), "\n");
                cursor.down(doc);
            }
            (Insert, BACKSPACE) => {
                if cursor.row == 0 && cursor.col == 0 {
                    return Ok(true);
                }

                if cursor.col == 0 {
                    cursor.up().go_to_end_of_line(doc, Insert);
                    doc.delete(cursor.to_pos(), cursor.to_pos());
                    return Ok(true);
                }

                cursor.bound_col(doc, *mode).left();
                doc.delete(cursor.to_pos(), cursor.to_pos());
            }
            (Insert, ch) if ch.is_control() => {}
            (Insert, ch) => {
                doc.insert(cursor.to_pos(), &ch.to_string());
                cursor.right(doc, *mode);
            }
            _ => {}
        }

        Ok(true)
    }
}
