use crate::screen::{
    Cell, Context, Mode, backspace, break_line, move_block_horizontally, move_block_vertically,
};
use std::{
    fs,
    io::{Read, stdin},
};

pub fn exec_binding(context: &mut Context, key: char) -> anyhow::Result<bool> {
    let Context {
        cursor,
        back_buffer,
        front_buffer,
        mode,
        ..
    } = context;

    match (mode, key) {
        (Mode::Normal, 'Q') => return Ok(false),
        (Mode::Normal, 'W') => {
            let mut content = String::new();
            let mut i = 0;
            while i < front_buffer.width * front_buffer.height {
                if front_buffer.cells[i].char == ' ' {
                    i += front_buffer.width - i % front_buffer.width;
                    content.push('\n');
                    continue;
                }

                if front_buffer.cells[i].char == '·' {
                    front_buffer.cells[i].char = ' ';
                }

                content.push(front_buffer.cells[i].char);
                i += 1;
            }

            // fs::write(&format!("{}.copy", &context.file_path), content.clone())?;
            fs::write(context.file_path.clone(), content)?;
        }
        (Mode::Normal, 'h') => cursor.left(back_buffer, context.mode),
        (Mode::Normal, 'j') => cursor.down(back_buffer),
        (Mode::Normal, 'k') => cursor.up(back_buffer),
        (Mode::Normal, 'l') => cursor.right(back_buffer, context.mode),
        (Mode::Normal, 'i') => {
            context.mode = Mode::Insert;
            cursor.reset(back_buffer, context.mode);
        }
        (Mode::Normal, 'u') => {
            if let Some(undo) = context.undo_list.pop() {
                for u in undo.2.iter() {
                    let idx = u.1 * back_buffer.width + u.0;
                    back_buffer.cells[idx].char = u.2;
                }

                cursor.x = undo.0;
                cursor.y = undo.1;
            }

            context.mode = Mode::Undo;
        }
        (Mode::Normal, 'I') => {
            context.mode = Mode::Insert;
            cursor.go_to_line_start(back_buffer, context.mode);
        }
        (Mode::Normal, 'w') => cursor.go_to_next_word(back_buffer),
        (Mode::Normal, 'b') => cursor.go_to_prev_word(back_buffer),
        (Mode::Normal, 'e') => cursor.go_to_last_char_of_next_word(back_buffer),
        (Mode::Normal, 'A') => {
            context.mode = Mode::Insert;
            cursor.go_to_line_end(back_buffer, context.mode);
        }
        (Mode::Normal, 's') => {
            context.mode = Mode::Insert;
            cursor.right(back_buffer, context.mode);
            backspace(back_buffer, cursor).unwrap();
        }
        (Mode::Normal, 'a') => {
            context.mode = Mode::Insert;
            cursor.right(back_buffer, context.mode);
        }
        (Mode::Normal, 'o') => {
            if cursor.y < back_buffer.last_line() {
                move_block_vertically(
                    back_buffer,
                    cursor.y + 1,
                    back_buffer.last_line() - cursor.y,
                    1,
                )
                .unwrap();
            }

            cursor.x = 0;
            cursor.y += 1;
            context.mode = Mode::Insert;
        }
        (Mode::Replace, char) => {
            let i = context.cursor.y * context.back_buffer.width + context.cursor.x;

            if !char.is_control() {
                context.back_buffer.cells[i] = Cell { char: key };
            }

            context.mode = Mode::Normal;
        }
        (Mode::Normal, 'r') => {
            context.mode = Mode::Replace;
        }
        (Mode::Insert, '\x1b') => {
            context.mode = Mode::Normal;
            cursor.reset(back_buffer, context.mode);
        }
        (Mode::Delete, 'd') => {
            move_block_vertically(
                back_buffer,
                cursor.y + 1,
                back_buffer.last_line() - cursor.y,
                -1,
            )
            .unwrap();

            context.mode = Mode::Normal;
            cursor.reset(back_buffer, context.mode);
        }
        (Mode::Delete, 'j') => {
            move_block_vertically(
                back_buffer,
                cursor.y + 2,
                back_buffer.last_line() - cursor.y - 1,
                -2,
            )
            .unwrap();

            context.mode = Mode::Normal;
            cursor.reset(back_buffer, context.mode);
        }
        (Mode::Delete, 'k') => {
            move_block_vertically(
                back_buffer,
                cursor.y + 1,
                back_buffer.last_line() - cursor.y - 1,
                -2,
            )
            .unwrap();

            context.mode = Mode::Normal;
            cursor.y -= 1;
            cursor.reset(back_buffer, context.mode);
        }
        (Mode::Delete, _) => {
            context.mode = Mode::Normal;
        }
        (Mode::Normal, 'd') => {
            context.mode = Mode::Delete;
        }
        (Mode::Normal, 'g') => {
            let mut k = [0; 1];
            stdin().read_exact(&mut k)?;

            if k[0] as char != 'g' {
                return exec_binding(context, k[0] as char);
            }

            cursor.y = 0;
            cursor.x = 0;
        }
        (Mode::Normal, 'G') => {
            cursor.y = back_buffer.last_line();
            cursor.go_to_line_end(back_buffer, context.mode);
        }
        (Mode::Insert, '\x08' | '\x7F') => backspace(back_buffer, cursor).unwrap(),
        (Mode::Insert, '\n' | '\r') => {
            break_line(back_buffer, cursor.x, cursor.y).unwrap();
            cursor.x = 0;
            cursor.y += 1;
        }
        (Mode::Insert, char) if char.is_control() => {}
        (Mode::Insert, mut char) => {
            move_block_horizontally(
                back_buffer,
                cursor.x,
                cursor.y,
                back_buffer.last_char(cursor.y) + 1 - cursor.x,
                1,
            )
            .unwrap();

            let idx = cursor.y * back_buffer.width + cursor.x;

            if char == ' ' {
                char = '·'
            }

            back_buffer.cells[idx] = Cell { char };
            cursor.right(back_buffer, context.mode);
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
