use std::io::{self, Read, stdin};

use crate::{BACKSPACE, ENTER, ESC, cursor::Cursor, document::Document, viewport::Viewport};

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    Replace,
    Delete,
    Insert,
    Visual(usize),
    Undo,
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

        let App {
            cursor, doc, mode, ..
        } = self;

        match (*mode, key) {
            (Normal | Visual(_), 'Q') => return Ok(false),
            (Normal | Visual(_), 'W') => doc.save()?,
            (Normal | Visual(_), 'h') => {
                cursor.left(doc);
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
                cursor.bound_col(doc, mode.set(Insert));
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
            (Normal, 'I') => {
                mode.set(Insert);
                cursor.go_to_start_of_line(doc);
            }
            // (Mode::Normal | Mode::Visual(_), 'w') => cursor.go_to_next_word(screen),
            // (Mode::Normal | Mode::Visual(_), 'b') => cursor.go_to_prev_word(screen),
            // (Mode::Normal | Mode::Visual(_), 'e') => cursor.go_to_last_char_of_next_word(screen),
            (Normal, 'A') => {
                cursor.go_to_end_of_line(doc, mode.set(Insert));
            }
            (Normal, 's') => {
                mode.set(Insert);
                doc.delete(cursor.to_pos(), cursor.to_pos());
            }
            (Normal, 'a') => {
                mode.set(Insert);
                cursor.right(doc, self.mode);
            }
            (Normal, 'o') => {
                mode.set(Insert);
                doc.insert_line(cursor.row + 1);
                cursor.down(doc);
            }
            (Mode::Replace, ch) => {
                if !ch.is_control() {
                    doc.delete(cursor.to_pos(), cursor.to_pos());
                    doc.insert(cursor.to_pos(), &ch.to_string());
                }

                mode.set(Normal);
            }
            (Normal, 'r') => {
                mode.set(Replace);
            }
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
            (Insert, ESC) => {
                cursor.bound_col(doc, mode.set(Normal));
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
            (Normal, 'g') => {
                let key = get_key_pressed()?;

                if key != 'g' {
                    return self.handle_input(key);
                }

                cursor.go_to_first_line();
            }
            (Normal | Mode::Visual(_), 'G') => {
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

                cursor.left(doc);
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

                cursor.left(doc);
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
