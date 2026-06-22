use std::{
    cmp::min,
    env,
    io::{Read, stdin},
    process::{Command, exit},
};

use ti::{
    screen::{Cell, Context, Mode, ScreenBuffer},
    *,
};

fn generate_diff(front: &ScreenBuffer, back: &ScreenBuffer) -> Vec<(usize, usize, char)> {
    let mut diff = Vec::new();

    for i in 0..front.cells.len() {
        if front.cells[i] != back.cells[i] {
            let x = i % front.width;
            let y = i / front.width;
            diff.push((x, y, back.cells[i].char));
        }
    }

    diff
}

fn generate_patch(diff: Vec<(usize, usize, char)>, context: &Context) -> String {
    let mut render = String::new();

    render.push_str(HIDE_CURSOR);

    for (cx, cy, char) in diff {
        render.push_str(&format!("\x1b[{};{}H{}", cy + 1, cx + 1, char));
    }

    render.push_str(&context.cursor.build(Some(context.lines[context.cursor.y])));

    render.push_str(SHOW_CURSOR);

    render
}

fn backspace(context: &mut Context, times: usize) {
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

fn process_input(context: &mut Context) -> anyhow::Result<bool> {
    let mut key = [0; 1];
    stdin().read_exact(&mut key)?;

    match (context.mode, key[0] as char) {
        (Mode::Normal, 'q') => return Ok(false),
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
            context.cursor.block = false;
            context.cursor.x = min(context.lines[context.cursor.y], context.cursor.x);
        }
        (Mode::Normal, 's') => {
            context.mode = Mode::Insert;
            context.cursor.block = false;
            context.cursor.x = min(context.lines[context.cursor.y], context.cursor.x) + 1;
            backspace(context, 1);
        }
        (Mode::Normal, 'a') => {
            context.mode = Mode::Insert;
            context.cursor.block = false;
            context.cursor.x = min(context.lines[context.cursor.y], context.cursor.x) + 1;
        }

        (Mode::Insert, '\x1B') => {
            context.mode = Mode::Normal;
            context.cursor.block = true;
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
        (Mode::Insert, char) => {
            let idx = context.cursor.y * context.back_buffer.width + context.cursor.x;
            let end_line = (context.cursor.y + 1) * context.back_buffer.width;

            context
                .back_buffer
                .cells
                .copy_within(idx..(end_line - 1), idx + 1);

            context.back_buffer.cells[idx] = Cell { char };
            context.cursor.x += 1;
        }
        _ => {}
    }

    Ok(true)
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("You need to provide the file path!");
        exit(1);
    }

    let mut context = Context::new(&args[1]);

    Command::new("stty").args(["raw", "-echo"]).status()?;

    print!("{CLEAR_SCREEN}");
    context.front_buffer.print();
    prin!("{}", context.cursor.build(None));

    loop {
        if !process_input(&mut context)? {
            break;
        }

        let diff = generate_diff(&context.front_buffer, &context.back_buffer);

        let patch = generate_patch(diff, &context);

        context.front_buffer = context.back_buffer.clone();

        prin!("{patch}");
    }

    print!("{CLEAR_SCREEN}");

    Command::new("stty").arg("sane").status()?;

    Ok(())
}
