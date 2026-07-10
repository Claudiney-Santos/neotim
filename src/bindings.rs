use std::{
    cmp::max,
    fs,
    io::{Read, stdin},
};

use crate::{cursor::Cursor, screen::ScreenBuffer};

use super::screen::{Cell, Context, Mode};

pub fn move_block_horizontally(
    screen: &mut ScreenBuffer,
    x: usize,
    y: usize,
    size: usize,
    steps: isize,
) -> Result<(), String> {
    if (steps < 0 && x as isize + steps < 0)
        && (steps >= 0 && x + size + steps as usize >= screen.width)
    {
        return Err(String::from("There is no space to do that action"));
    }

    let start = y * screen.width + x;
    let end = y * screen.width + x + size;

    screen
        .cells
        .copy_within(start..end, max(0, start as isize + steps) as usize);

    if steps >= 0 {
        for i in start..start + steps as usize {
            screen.cells[i] = Cell { char: '·' };
        }
    } else {
        for i in max(0, end as isize + steps) as usize..end {
            screen.cells[i] = Cell { char: ' ' };
        }
    }

    Ok(())
}

pub fn move_block_vertically(
    screen: &mut ScreenBuffer,
    line: usize,
    size: usize,
    steps: isize,
) -> Result<(), String> {
    if (steps < 0 && line as isize + steps < 0)
        || (steps >= 0 && (line + size) as isize >= screen.height as isize - steps)
    {
        return Err(String::from("There is no space to do that action"));
    }

    let start = line * screen.width;
    let end = (line + size) * screen.width;

    let dest = max(0, start as isize + (screen.width as isize * steps)) as usize;

    screen.cells.copy_within(start..(end - 1), dest);

    if steps < 0 {
        for i in max(0, end as isize + steps * screen.width as isize) as usize..end {
            screen.cells[i] = Cell { char: ' ' };
        }
    } else {
        for i in start..((line + max(0, steps as usize)) * screen.width) {
            screen.cells[i] = Cell { char: ' ' };
        }
    }

    Ok(())
}

pub fn break_line(screen: &mut ScreenBuffer, x: usize, y: usize) -> Result<(), String> {
    move_block_vertically(screen, y + 1, screen.last_line() + 1 - y, 1)?;

    let start = y * screen.width + x;
    let end = y * screen.width + screen.line_len(y);

    screen.cells.copy_within(start..end, (y + 1) * screen.width);

    for i in start..end {
        screen.cells[i] = Cell { char: ' ' };
    }

    Ok(())
}

pub fn backspace(screen: &mut ScreenBuffer, cursor: &mut Cursor) -> Result<(), String> {
    if cursor.x == 0 && cursor.y == 0 {
        return Ok(());
    }

    if cursor.x > 0 {
        move_block_horizontally(
            screen,
            cursor.x,
            cursor.y,
            screen.last_char(cursor.y) - cursor.x,
            -1,
        )?;
        cursor.x -= 1;
        return Ok(());
    }

    let start = cursor.y * screen.width;
    let end = cursor.y * screen.width + screen.last_char(cursor.y);

    let dest = (cursor.y - 1) * screen.width + screen.last_char(cursor.y - 1);

    screen.cells.copy_within(start..end, dest);

    for i in start..end {
        screen.cells[i] = Cell { char: ' ' };
    }

    move_block_vertically(screen, cursor.y + 1, screen.last_line() - cursor.y - 1, -1)?;

    cursor.x = screen.last_char(cursor.y - 1);
    cursor.y -= 1;

    Ok(())
}

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
