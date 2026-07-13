use crate::{
    BACKSPACE, ENTER, ESC,
    cursor::y_bounded,
    file,
    screen::{
        Cell, Context, Mode, backspace, break_line, move_block_horizontally, move_block_vertically,
    },
};
use std::{
    cmp::{max, min},
    io::{Read, stdin},
};

pub fn exec_binding(context: &mut Context, key: char) -> anyhow::Result<bool> {
    let Context {
        cursor,
        file_path,
        back_buffer: screen,
        mode,
        ..
    } = context;

    match (mode, key) {
        (Mode::Normal | Mode::Visual(_), 'Q') => return Ok(false),
        (Mode::Normal | Mode::Visual(_), 'W') => file::save(file_path, screen)?,
        (Mode::Normal | Mode::Visual(_), 'h') => cursor.left(screen, context.mode),
        (Mode::Normal | Mode::Visual(_), 'j') => cursor.down(screen),
        (Mode::Normal | Mode::Visual(_), 'k') => cursor.up(screen),
        (Mode::Normal | Mode::Visual(_), 'l') => cursor.right(screen, context.mode),
        (Mode::Normal, 'i') => {
            context.mode = Mode::Insert;
            cursor.reset(screen, context.mode);
        }
        (Mode::Normal, 'u') => {
            context.undo_stack.pop().map(|mut undo| {
                undo.delta.reverse();
                for (x, y, cell) in undo.delta.iter() {
                    screen.cells[y * screen.width + x] = *cell;
                }
                *cursor = undo.cursor;
                screen.line_count = undo.line_count;

                context.mode = Mode::Undo;
            });
        }
        (Mode::Normal, 'I') => {
            context.mode = Mode::Insert;
            cursor.go_to_line_start(screen);
        }
        (Mode::Normal | Mode::Visual(_), 'w') => cursor.go_to_next_word(screen),
        (Mode::Normal | Mode::Visual(_), 'b') => cursor.go_to_prev_word(screen),
        (Mode::Normal | Mode::Visual(_), 'e') => cursor.go_to_last_char_of_next_word(screen),
        (Mode::Normal, 'A') => {
            context.mode = Mode::Insert;
            cursor.go_to_line_end(screen, context.mode);
        }
        (Mode::Normal, 's') => {
            context.mode = Mode::Insert;
            cursor.right(screen, context.mode);
            backspace(screen, cursor)?;
        }
        (Mode::Normal, 'a') => {
            context.mode = Mode::Insert;
            cursor.right(screen, context.mode);
        }
        (Mode::Normal, 'o') => {
            if cursor.y < screen.line_count {
                move_block_vertically(screen, cursor.y + 1, screen.line_count - cursor.y, 1)?;
            }

            cursor.x = 0;
            cursor.y += 1;
            context.mode = Mode::Insert;
        }
        (Mode::Replace, char) => {
            let i = cursor.y * screen.width + cursor.x;

            if !char.is_control() {
                screen.cells[i] = Cell::new(key);
            }

            context.mode = Mode::Normal;
        }
        (Mode::Normal, 'r') => {
            context.mode = Mode::Replace;
        }
        (Mode::Normal, 'x') => {
            if screen.line_len(cursor.y) > 0 {
                cursor.right(screen, Mode::Insert);
                backspace(screen, cursor)?;
            }
        }
        (Mode::Normal, 'v') => {
            context.mode = Mode::Visual(cursor.y * screen.width + cursor.x);
        }
        (Mode::Visual(landmark), ESC) => {
            let idx = cursor.y * screen.width + cursor.x;
            let start = min(idx, *landmark);
            let end = max(idx, *landmark);

            for i in start..end {
                screen.cells[i].highlight = false;
            }
            context.mode = Mode::Normal;
            cursor.reset(screen, context.mode);
        }
        (Mode::Insert, ESC) => {
            context.mode = Mode::Normal;
            cursor.reset(screen, context.mode);
        }
        (Mode::Delete, 'd') => {
            move_block_vertically(screen, cursor.y + 1, screen.line_count - cursor.y, -1)?;

            context.mode = Mode::Normal;
            cursor.reset(screen, context.mode);
        }
        (Mode::Delete, 'j') => {
            move_block_vertically(
                screen,
                y_bounded(cursor.y as isize + 2, screen),
                screen.line_count - cursor.y - 1,
                -2,
            )?;

            context.mode = Mode::Normal;
            cursor.reset(screen, context.mode);
        }
        (Mode::Delete, 'k') => {
            move_block_vertically(screen, cursor.y + 1, screen.line_count - cursor.y - 1, -2)?;

            context.mode = Mode::Normal;
            cursor.y -= 1;
            cursor.reset(screen, context.mode);
        }
        (Mode::Delete, _) => {
            context.mode = Mode::Normal;
        }
        (Mode::Normal, 'd') => {
            context.mode = Mode::Delete;
        }
        (Mode::Normal | Mode::Visual(_), 'g') => {
            let mut k = [0; 1];
            stdin().read_exact(&mut k)?;

            if k[0] as char != 'g' {
                return exec_binding(context, k[0] as char);
            }

            cursor.y = 0;
            cursor.x = 0;
        }
        (Mode::Normal | Mode::Visual(_), 'G') => {
            cursor.y = screen.line_count - 1;
            cursor.go_to_line_end(screen, context.mode);
        }
        (Mode::Normal, ENTER) => {
            cursor.down(screen);
            cursor.go_to_line_start(screen);
        }
        (Mode::Normal, BACKSPACE) => {
            if cursor.x == 0 {
                cursor.up(screen);
                cursor.go_to_line_end(screen, context.mode);
            } else {
                cursor.left(screen, context.mode);
            }
        }
        (Mode::Insert, ENTER) => {
            break_line(screen, cursor.x, cursor.y)?;
            cursor.x = 0;
            cursor.y += 1;
        }
        (Mode::Insert, BACKSPACE) => backspace(screen, cursor)?,
        (Mode::Insert, ch) if ch.is_control() => {}
        (Mode::Insert, ch) => {
            move_block_horizontally(
                screen,
                cursor.x,
                cursor.y,
                screen.line_len(cursor.y) - cursor.x,
                1,
            )?;

            let idx = cursor.y * screen.width + cursor.x;
            let char = match ch {
                ' ' => '·',
                ch => ch,
            };

            screen.cells[idx] = Cell::new(char);
            cursor.right(screen, context.mode);
        }
        _ => {}
    }

    Ok(true)
}

pub fn process_input(context: &mut Context) -> anyhow::Result<bool> {
    let mut key = [0; 1];
    stdin().read_exact(&mut key)?;

    exec_binding(context, key[0] as char)
}
