use std::{
    cmp::{max, min},
    fs,
    io::{Read, stdin},
};

use super::screen::{Cell, Context, Mode};

pub fn move_block_horizontally(
    context: &mut Context,
    x: usize,
    y: usize,
    size: usize,
    steps: isize,
) -> Result<(), String> {
    if (steps < 0 && x as isize + steps < 0)
        && (steps >= 0 && x + size + steps as usize >= context.back_buffer.width)
    {
        return Err(String::from("There is no space to do that action"));
    }

    let start = y * context.back_buffer.width + x;
    let end = y * context.back_buffer.width + x + size;

    context
        .back_buffer
        .cells
        .copy_within(start..end, max(0, start as isize + steps) as usize);

    if steps >= 0 {
        for i in start..start + steps as usize {
            context.back_buffer.cells[i] = Cell { char: '·' };
        }
    } else {
        for i in max(0, end as isize + steps) as usize..end {
            context.back_buffer.cells[i] = Cell { char: ' ' };
        }
    }

    Ok(())
}

pub fn move_block_vertically(
    context: &mut Context,
    line: usize,
    size: usize,
    steps: isize,
) -> Result<(), String> {
    if (steps < 0 && line as isize + steps < 0)
        || (steps >= 0 && (line + size) as isize >= context.back_buffer.height as isize - steps)
    {
        return Err(String::from("There is no space to do that action"));
    }

    let start = line * context.back_buffer.width;
    let end = (line + size) * context.back_buffer.width;

    let dest = max(
        0,
        start as isize + (context.back_buffer.width as isize * steps),
    ) as usize;

    context
        .back_buffer
        .cells
        .copy_within(start..(end - 1), dest);

    if steps < 0 {
        for i in max(0, end as isize + steps * context.back_buffer.width as isize) as usize..end {
            context.back_buffer.cells[i] = Cell { char: ' ' };
        }
    } else {
        for i in start..((line + max(0, steps as usize)) * context.back_buffer.width) {
            context.back_buffer.cells[i] = Cell { char: ' ' };
        }
    }

    Ok(())
}

pub fn break_line(context: &mut Context, x: usize, y: usize) -> Result<(), String> {
    move_block_vertically(context, y + 1, context.back_buffer.last_line() - y, 1)?;

    let start = y * context.back_buffer.width + x;
    let end = y * context.back_buffer.width + context.back_buffer.last_char(y);

    context
        .back_buffer
        .cells
        .copy_within(start..end, (y + 1) * context.back_buffer.width);

    for i in start..end {
        context.back_buffer.cells[i] = Cell { char: ' ' };
    }

    Ok(())
}

pub fn backspace(context: &mut Context) -> Result<(), String> {
    if context.cursor.x == 0 && context.cursor.y == 0 {
        return Ok(());
    }

    if context.cursor.x > 0 {
        move_block_horizontally(
            context,
            context.cursor.x,
            context.cursor.y,
            context.back_buffer.last_char(context.cursor.y) - context.cursor.x,
            -1,
        )?;
        context.cursor.x -= 1;
        return Ok(());
    }

    let start = context.cursor.y * context.back_buffer.width;
    let end = context.cursor.y * context.back_buffer.width
        + context.back_buffer.last_char(context.cursor.y);

    let dest = (context.cursor.y - 1) * context.back_buffer.width
        + context.back_buffer.last_char(context.cursor.y - 1);

    context.back_buffer.cells.copy_within(start..end, dest);

    for i in start..end {
        context.back_buffer.cells[i] = Cell { char: ' ' };
    }

    move_block_vertically(
        context,
        context.cursor.y + 1,
        context.back_buffer.last_line() - context.cursor.y - 1,
        -1,
    )?;

    context.cursor.x = context.back_buffer.last_char(context.cursor.y - 1);
    context.cursor.y -= 1;

    Ok(())
}

pub fn exec_binding(context: &mut Context, key: char) -> anyhow::Result<bool> {
    match (context.mode, key) {
        (Mode::Normal, 'Q') => return Ok(false),
        (Mode::Normal, 'W') => {
            let mut content = String::new();
            let mut i = 0;
            while i < context.front_buffer.width * context.front_buffer.height {
                if context.front_buffer.cells[i].char == ' ' {
                    i += context.front_buffer.width - i % context.front_buffer.width;
                    content.push('\n');
                    continue;
                }

                if context.front_buffer.cells[i].char == '·' {
                    context.front_buffer.cells[i].char = ' ';
                }

                content.push(context.front_buffer.cells[i].char);
                i += 1;
            }

            fs::write(&format!("{}.copy", &context.file_path), content.clone())?;
            fs::write(context.file_path.clone(), content)?;
        }
        (Mode::Normal, 'h') if context.cursor.x as i32 > 0 => {
            context.cursor.x = context.get_min_x() - 1;
        }
        (Mode::Normal, 'j')
            if context.cursor.y + 1
                < min(context.back_buffer.last_line(), context.back_buffer.height) =>
        {
            context.cursor.y += 1
        }
        (Mode::Normal, 'k') if context.cursor.y as i32 > 0 => context.cursor.y -= 1,
        (Mode::Normal, 'l')
            if context.cursor.x + 1
                < min(
                    context.back_buffer.last_char(context.cursor.y),
                    context.back_buffer.width,
                ) =>
        {
            context.cursor.x += 1
        }
        (Mode::Normal, 'i') => {
            context.mode = Mode::Insert;
            context.cursor.x = context.get_min_x();
        }
        (Mode::Normal, 'I') => {
            context.mode = Mode::Insert;
            context.cursor.x = 0;

            let start = context.cursor.y * context.back_buffer.width;
            let end = start + context.back_buffer.last_char(context.cursor.y);

            for i in start..end {
                if context.back_buffer.cells[i].char != ' ' {
                    context.cursor.x = i % context.back_buffer.width;
                    break;
                }
            }
        }
        (Mode::Normal, 'w') => {
            let mut idx = context.cursor.y * context.back_buffer.width + context.cursor.x;

            enum State {
                Alphabetic,
                NonAlphabetic,
                WhiteSpace,
            }

            let state = match context.back_buffer.cells[idx].char {
                c if c.is_alphanumeric() => State::Alphabetic,
                c if c.is_ascii_whitespace() => State::WhiteSpace,
                _ => State::NonAlphabetic,
            };

            for (i, c) in context.back_buffer.cells.iter().skip(idx).enumerate() {
                match (&state, c.char) {
                    (State::Alphabetic, c) if !c.is_alphanumeric() => {
                        idx += i;
                        break;
                    }
                    (State::NonAlphabetic, c) if c.is_alphanumeric() || c.is_whitespace() => {
                        idx += i;
                        break;
                    }
                    (State::WhiteSpace, c) if !c.is_whitespace() => {
                        idx += i;
                        break;
                    }
                    _ => {}
                }
            }

            for (i, c) in context.back_buffer.cells.iter().skip(idx).enumerate() {
                match c.char {
                    c if !c.is_whitespace() => {
                        idx += i;
                        break;
                    }
                    _ => {}
                }
            }

            context.cursor.x = idx % context.back_buffer.width;
            context.cursor.y = idx / context.back_buffer.width;
        }
        (Mode::Normal, 'b') => {
            let mut idx = context.cursor.y * context.back_buffer.width + context.cursor.x;

            enum State {
                Alphabetic,
                NonAlphabetic,
                WhiteSpace,
            }

            let mut state = match context.back_buffer.cells[idx - 1].char {
                c if c.is_alphanumeric() => State::Alphabetic,
                c if c.is_ascii_whitespace() => State::WhiteSpace,
                _ => State::NonAlphabetic,
            };

            for (i, c) in context
                .back_buffer
                .cells
                .iter()
                .take(idx - 1)
                .rev()
                .enumerate()
            {
                match (&state, c.char) {
                    (State::Alphabetic, c) if !c.is_alphanumeric() => {
                        idx -= i + 1;
                        break;
                    }
                    (State::NonAlphabetic, c) if c.is_alphanumeric() || c.is_whitespace() => {
                        idx -= i + 1;
                        break;
                    }
                    (State::WhiteSpace, c) if !c.is_whitespace() => {
                        if c.is_alphanumeric() {
                            state = State::Alphabetic;
                        } else {
                            state = State::NonAlphabetic;
                        }
                    }
                    _ => {}
                }
            }

            context.cursor.x = idx % context.back_buffer.width;
            context.cursor.y = idx / context.back_buffer.width;
        }
        (Mode::Normal, 'e') => {
            let mut idx = context.cursor.y * context.back_buffer.width + context.cursor.x;

            enum State {
                Alphabetic,
                NonAlphabetic,
                WhiteSpace,
            }

            let mut state = match context.back_buffer.cells[idx + 1].char {
                c if c.is_alphanumeric() => State::Alphabetic,
                c if c.is_ascii_whitespace() => State::WhiteSpace,
                _ => State::NonAlphabetic,
            };

            for (i, c) in context.back_buffer.cells.iter().skip(idx + 1).enumerate() {
                match (&state, c.char) {
                    (State::Alphabetic, c) if !c.is_alphanumeric() => {
                        idx += i;
                        break;
                    }
                    (State::NonAlphabetic, c) if c.is_alphanumeric() || c.is_whitespace() => {
                        idx += i;
                        break;
                    }
                    (State::WhiteSpace, c) if !c.is_whitespace() => {
                        if c.is_alphanumeric() {
                            state = State::Alphabetic;
                        } else {
                            state = State::NonAlphabetic;
                        }
                    }
                    _ => {}
                }
            }

            context.cursor.x = idx % context.back_buffer.width;
            context.cursor.y = idx / context.back_buffer.width;
        }
        (Mode::Normal, 'A') => {
            context.mode = Mode::Insert;
            context.cursor.x = context.get_min_x();
        }
        (Mode::Normal, 's') => {
            context.mode = Mode::Insert;
            context.cursor.x = context.get_min_x() + 1;
            backspace(context).unwrap();
        }
        (Mode::Normal, 'a') => {
            context.mode = Mode::Insert;
            context.cursor.x = context.get_min_x() + 1;
        }
        (Mode::Normal, 'o') => {
            move_block_vertically(
                context,
                context.cursor.y + 1,
                context.back_buffer.last_line() - context.cursor.y,
                1,
            )
            .unwrap();

            context.cursor.x = 0;
            context.cursor.y += 1;
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
            context.cursor.x = context.get_min_x();
            if context.cursor.x > 0 {
                context.cursor.x -= 1;
            }
        }
        (Mode::Delete, 'd') => {
            move_block_vertically(
                context,
                context.cursor.y + 1,
                context.back_buffer.last_line() - context.cursor.y,
                -1,
            )
            .unwrap();

            context.cursor.x = context.get_min_x();
            context.mode = Mode::Normal;
        }
        (Mode::Delete, 'j') => {
            move_block_vertically(
                context,
                context.cursor.y + 2,
                context.back_buffer.last_line() - context.cursor.y - 1,
                -2,
            )
            .unwrap();

            context.cursor.x = context.get_min_x();
            context.mode = Mode::Normal;
        }
        (Mode::Delete, 'k') => {
            move_block_vertically(
                context,
                context.cursor.y + 1,
                context.back_buffer.last_line() - context.cursor.y - 1,
                -2,
            )
            .unwrap();

            context.cursor.y -= 1;
            context.cursor.x = context.get_min_x();
            context.mode = Mode::Normal;
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

            context.cursor.y = 0;
            context.cursor.x = 0;
        }
        (Mode::Normal, 'G') => {
            context.cursor.y = context.get_min_y();
            context.cursor.x = context.get_min_x();
        }
        (Mode::Insert, '\x08' | '\x7F') => backspace(context).unwrap(),
        (Mode::Insert, '\n' | '\r') => {
            break_line(context, context.cursor.x, context.cursor.y).unwrap();
            context.cursor.x = 0;
            context.cursor.y += 1;
        }
        (Mode::Insert, char) if char.is_control() => {}
        (Mode::Insert, char) => {
            move_block_horizontally(
                context,
                context.cursor.x,
                context.cursor.y,
                context.back_buffer.last_char(context.cursor.y) - context.cursor.x,
                1,
            )
            .unwrap();

            let idx = context.cursor.y * context.back_buffer.width + context.cursor.x;

            context.back_buffer.cells[idx] = Cell { char };
            context.cursor.x += 1;
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
