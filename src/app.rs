use std::io::{self, Read, stdin};

use crate::{BACKSPACE, ESC, cursor::Cursor, document::Document, viewport::Viewport};

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    Replace,
    Delete,
    Insert,
    Visual(usize),
    Undo,
}

pub struct App {
    pub doc: Document,
    pub viewport: Viewport,
    pub cursor: Cursor,
    pub mode: Mode,
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
        })
    }

    pub fn handle_input(&mut self, key: char) -> anyhow::Result<bool> {
        use Mode::*;

        let App { cursor, doc, .. } = self;

        match (self.mode, key) {
            (Normal | Visual(_), 'Q') => return Ok(false),
            (Normal | Visual(_), 'W') => doc.save()?,
            (Normal | Visual(_), 'h') => cursor.left(doc, self.mode),
            (Normal | Visual(_), 'j') => cursor.down(doc, self.mode),
            (Normal | Visual(_), 'k') => cursor.up(doc, self.mode),
            (Normal | Visual(_), 'l') => cursor.right(doc, self.mode),
            (Mode::Normal, 'i') => {
                self.mode = Mode::Insert;
                (cursor.col, _) =
                    Cursor::bound(cursor.col as isize, cursor.row as isize, doc, self.mode);
            }
            // (Mode::Normal, 'u') => {
            //     context.undo_stack.pop().map(|mut undo| {
            //         undo.delta.reverse();
            //         for (x, y, cell) in undo.delta.iter() {
            //             screen.cells[y * screen.width + x] = *cell;
            //         }
            //         *cursor = undo.cursor;
            //         screen.line_count = undo.line_count;
            //
            //         context.mode = Mode::Undo;
            //     });
            // }
            (Mode::Normal, 'I') => {
                self.mode = Mode::Insert;
                cursor.go_to_start_of_line(doc);
            }
            // (Mode::Normal | Mode::Visual(_), 'w') => cursor.go_to_next_word(screen),
            // (Mode::Normal | Mode::Visual(_), 'b') => cursor.go_to_prev_word(screen),
            // (Mode::Normal | Mode::Visual(_), 'e') => cursor.go_to_last_char_of_next_word(screen),
            (Mode::Normal, 'A') => {
                self.mode = Mode::Insert;
                cursor.go_to_end_of_line(doc, self.mode);
            }
            // (Mode::Normal, 's') => {
            //     context.mode = Mode::Insert;
            //     cursor.right(screen, context.mode);
            //     backspace(screen, cursor)?;
            // }
            (Mode::Normal, 'a') => {
                self.mode = Mode::Insert;
                cursor.right(doc, self.mode);
            }
            // (Mode::Normal, 'o') => {
            //     if cursor.y < screen.line_count {
            //         move_block_vertically(screen, cursor.y + 1, screen.line_count - cursor.y, 1)?;
            //     }
            //
            //     cursor.x = 0;
            //     cursor.y += 1;
            //     context.mode = Mode::Insert;
            // }
            // (Mode::Replace, char) => {
            //     let i = cursor.y * screen.width + cursor.x;
            //
            //     if !char.is_control() {
            //         screen.cells[i] = Cell::new(key);
            //     }
            //
            //     context.mode = Mode::Normal;
            // }
            // (Mode::Normal, 'r') => {
            //     context.mode = Mode::Replace;
            // }
            // (Mode::Normal, 'x') => {
            //     if screen.line_len(cursor.y) > 0 {
            //         cursor.right(screen, Mode::Insert);
            //         backspace(screen, cursor)?;
            //     }
            // }
            // (Mode::Normal, 'v') => {
            //     context.mode = Mode::Visual(cursor.y * screen.width + cursor.x);
            // }
            // (Mode::Visual(landmark), ESC) => {
            //     let idx = cursor.y * screen.width + cursor.x;
            //     let start = min(idx, *landmark);
            //     let end = max(idx, *landmark);
            //
            //     for i in start..end {
            //         screen.cells[i].highlight = false;
            //     }
            //     context.mode = Mode::Normal;
            //     cursor.reset(screen, context.mode);
            // }
            (Mode::Insert, ESC) => {
                self.mode = Mode::Normal;
                (cursor.col, _) =
                    Cursor::bound(cursor.col as isize, cursor.row as isize, doc, self.mode);
            }
            // (Mode::Visual(landmark), 'D') => {
            //     let cursor_raw = cursor.y * screen.width + cursor.x;
            //     let start = min(*landmark, cursor_raw);
            //     let end = max(*landmark, cursor_raw);
            //
            //     for i in start..end {
            //         screen.cells[i].highlight = false;
            //     }
            //
            //     let start = Pos::from_raw(start, screen.width);
            //     let end = Pos::from_raw(end, screen.width);
            //
            //     move_block_vertically(
            //         screen,
            //         end.y + 1,
            //         screen.line_count - (end.y + 1),
            //         start.y as isize - (end.y as isize + 1),
            //     )?;
            //
            //     *cursor = start.into();
            //     context.prev_cursor = start.into();
            //     context.mode = Mode::Normal;
            //     cursor.reset(screen, context.mode);
            // }
            // (Mode::Visual(landmark), 'd') => {
            //     let cursor_raw = cursor.y * screen.width + cursor.x;
            //     let start = min(*landmark, cursor_raw);
            //     let end = max(*landmark, cursor_raw);
            //
            //     for i in start..end {
            //         screen.cells[i].highlight = false;
            //     }
            //
            //     let content = cut(screen, end + 1, end + screen.width - (end % screen.width));
            //     paste(screen, start, content);
            //
            //     let start = Pos::from_raw(start, screen.width);
            //     let end = Pos::from_raw(end, screen.width);
            //
            //     move_block_vertically(
            //         screen,
            //         end.y + 1,
            //         screen.line_count - end.y,
            //         start.y as isize - end.y as isize,
            //     )?;
            //
            //     *cursor = start.into();
            //     context.prev_cursor = start.into();
            //     context.mode = Mode::Normal;
            //     cursor.reset(screen, context.mode);
            // }
            // (Mode::Delete, 'd') => {
            //     move_block_vertically(screen, cursor.y + 1, screen.line_count - cursor.y, -1)?;
            //
            //     context.mode = Mode::Normal;
            //     cursor.reset(screen, context.mode);
            // }
            // (Mode::Delete, 'j') => {
            //     move_block_vertically(
            //         screen,
            //         y_bounded(cursor.y as isize + 2, screen),
            //         screen.line_count - cursor.y - 1,
            //         -2,
            //     )?;
            //
            //     context.mode = Mode::Normal;
            //     cursor.reset(screen, context.mode);
            // }
            // (Mode::Delete, 'k') => {
            //     move_block_vertically(screen, cursor.y + 1, screen.line_count - cursor.y - 1, -2)?;
            //
            //     context.mode = Mode::Normal;
            //     cursor.y -= 1;
            //     cursor.reset(screen, context.mode);
            // }
            // (Mode::Delete, _) => {
            //     context.mode = Mode::Normal;
            // }
            // (Mode::Normal, 'd') => {
            //     context.mode = Mode::Delete;
            // }
            (Mode::Normal, 'g') => {
                let key = get_key_pressed()?;

                if key != 'g' {
                    return self.handle_input(key);
                }

                cursor.go_to_first_line();
            }
            (Mode::Normal | Mode::Visual(_), 'G') => {
                cursor.go_to_last_char(&doc);
            }
            // (Mode::Normal, ENTER) => {
            //     cursor.down(screen);
            //     cursor.go_to_line_start(screen);
            // }
            // (Mode::Normal, BACKSPACE) => {
            //     if cursor.x == 0 {
            //         cursor.up(screen);
            //         cursor.go_to_line_end(screen, context.mode);
            //     } else {
            //         cursor.left(screen, context.mode);
            //     }
            // }
            // (Mode::Insert, ENTER) => {
            //     break_line(screen, cursor.x, cursor.y)?;
            //     cursor.x = 0;
            //     cursor.y += 1;
            // }
            (Mode::Insert, BACKSPACE) => {
                cursor.left(doc, self.mode);
                doc.remove_char(cursor.col, cursor.row);
            }
            (Mode::Insert, ch) if ch.is_control() => {}
            (Mode::Insert, ch) => {
                doc.insert_char(cursor.col, cursor.row, ch);
                cursor.right(doc, self.mode);
            }
            _ => {}
        }

        Ok(true)
    }
}
