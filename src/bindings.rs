use std::{
    cmp::min,
    io::{Read, stdin},
};

use super::screen::{Cell, Context, CursorMode, Mode};

pub fn backspace(context: &mut Context, times: usize) {
    let idx = context.cursor.y * context.back_buffer.width + context.cursor.x;
    let end_line = (context.cursor.y + 1) * context.back_buffer.width;

    context
        .back_buffer
        .cells
        .copy_within(idx..(end_line - 1), idx - times);

    context.back_buffer.cells[end_line - 1] = Cell { char: ' ' };
    context.lines[context.cursor.y] -= 1;
    context.cursor.x -= times;
}

pub fn exec_binding(context: &mut Context, key: char) -> anyhow::Result<bool> {
    match (context.mode, key) {
        (Mode::Normal, 'Q') => return Ok(false),
        (Mode::Normal, 'h') if context.cursor.x as i32 > 0 => {
            context.cursor.x = min(context.lines[context.cursor.y], context.cursor.x) - 1;
        }
        (Mode::Normal, 'j') if context.cursor.y + 1 < context.lines.len() => context.cursor.y += 1,
        (Mode::Normal, 'k') if context.cursor.y as i32 > 0 => context.cursor.y -= 1,
        (Mode::Normal, 'l') if context.cursor.x + 1 < context.lines[context.cursor.y] => {
            context.cursor.x += 1
        }
        (Mode::Normal, 'i') => {
            context.mode = Mode::Insert;
            context.cursor.mode = CursorMode::Bar;
            context.cursor.x = min(context.lines[context.cursor.y], context.cursor.x);
        }
        (Mode::Normal, 'I') => {
            context.mode = Mode::Insert;
            context.cursor.mode = CursorMode::Bar;
            context.cursor.x = context.lines[context.cursor.y];
            for (i, c) in context
                .back_buffer
                .cells
                .iter()
                .skip(context.cursor.y * context.back_buffer.width)
                .take(context.lines[context.cursor.y])
                .enumerate()
            {
                if c.char != ' ' {
                    context.cursor.x = i;
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
            context.cursor.mode = CursorMode::Bar;
            context.cursor.x = context.lines[context.cursor.y];
        }
        (Mode::Normal, 's') => {
            context.mode = Mode::Insert;
            context.cursor.mode = CursorMode::Bar;
            context.cursor.x = min(context.lines[context.cursor.y], context.cursor.x) + 1;
            backspace(context, 1);
        }
        (Mode::Normal, 'a') => {
            context.mode = Mode::Insert;
            context.cursor.mode = CursorMode::Bar;
            context.cursor.x = min(context.lines[context.cursor.y], context.cursor.x) + 1;
        }
        (Mode::Replace, char) => {
            if !char.is_control() {
                context.back_buffer.cells
                    [context.cursor.y * context.back_buffer.width + context.cursor.x] =
                    Cell { char: key };
            }
            context.mode = Mode::Normal;
            context.cursor.mode = CursorMode::Block;
        }
        (Mode::Normal, 'r') => {
            context.mode = Mode::Replace;
            context.cursor.mode = CursorMode::Underline;
        }
        (Mode::Insert, '\t') => {
            context.mode = Mode::Normal;
            context.cursor.mode = CursorMode::Block;
            context.cursor.x = min(context.lines[context.cursor.y], context.cursor.x);
            if context.cursor.x > 0 {
                context.cursor.x -= 1;
            }
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
            context.cursor.y = context.lines.len() - 1;
            context.cursor.x = context.lines[context.cursor.y] - 1;
        }
        (Mode::Insert, '\x08' | '\x7F') => backspace(context, 1),
        (Mode::Insert, '\n' | '\r') if context.lines.len() < context.back_buffer.height - 1 => {
            let actual_line_idx = context.cursor.y * context.back_buffer.width;
            let next_line_idx = (context.cursor.y + 1) * context.back_buffer.width;
            let end_of_buffer = context.back_buffer.width * context.back_buffer.height;

            let idx = context.cursor.x + context.cursor.y * context.back_buffer.width;

            context.back_buffer.cells.copy_within(
                actual_line_idx..(end_of_buffer - context.back_buffer.width),
                next_line_idx,
            );

            for i in idx..next_line_idx {
                context.back_buffer.cells[i] = Cell { char: ' ' };
            }

            context.back_buffer.cells.copy_within(
                (next_line_idx + context.cursor.x)..(next_line_idx + context.back_buffer.width),
                next_line_idx,
            );

            for i in (next_line_idx + context.back_buffer.width - context.cursor.x)
                ..(next_line_idx + context.back_buffer.width)
            {
                context.back_buffer.cells[i] = Cell { char: ' ' };
            }

            context.lines.insert(
                context.cursor.y + 1,
                context.lines[context.cursor.y] - context.cursor.x,
            );
            context.lines[context.cursor.y] = context.cursor.x;

            context.cursor.x = 0;
            context.cursor.y += 1;
        }
        (Mode::Insert, char) if char.is_control() => {}
        (Mode::Insert, char) => {
            let idx = context.cursor.y * context.back_buffer.width + context.cursor.x;
            let end_line = (context.cursor.y + 1) * context.back_buffer.width;

            context
                .back_buffer
                .cells
                .copy_within(idx..(end_line - 1), idx + 1);

            context.back_buffer.cells[idx] = Cell { char };
            context.lines[context.cursor.y] += 1;
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
