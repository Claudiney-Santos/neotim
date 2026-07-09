use std::{
    cmp::min,
    env,
    process::{Command, exit},
};

use ti::{
    bindings::*,
    screen::{Context, Mode, ScreenBuffer},
    *,
};

fn generate_diff(
    front: &ScreenBuffer,
    back: &ScreenBuffer,
) -> (Vec<(usize, usize, char)>, Vec<(usize, usize, char)>) {
    let mut diff = Vec::new();
    let mut undo = Vec::new();

    for i in 0..front.cells.len() {
        if front.cells[i] != back.cells[i] {
            let x = i % front.width;
            let y = i / front.width;
            diff.push((x, y, back.cells[i].char));
            undo.push((x, y, front.cells[i].char));
        }
    }

    (diff, undo)
}

pub fn generate_patch(diff: Vec<(usize, usize, char)>, context: &Context) -> String {
    let mut render = String::new();

    render.push_str(HIDE_CURSOR);

    for (cx, cy, char) in diff {
        if char == '·' {
            render.push_str(&format!("\x1b[{};{}H\x1b[90m·\x1b[0m", cy + 1, cx + 1));
            continue;
        }

        render.push_str(&format!("\x1b[{};{}H{}", cy + 1, cx + 1, char));
    }

    let virtual_x = if context.mode == Mode::Insert {
        min(
            context.cursor.x,
            context.back_buffer.last_char(context.cursor.y) + 1,
        )
    } else {
        min(
            context.cursor.x,
            context.back_buffer.last_char(context.cursor.y),
        )
    };

    render.push_str(&context.cursor.build(virtual_x, context.mode));

    render.push_str(SHOW_CURSOR);

    render
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
    prin!("{}", context.cursor.build(0, context.mode));

    while process_input(&mut context)? {
        let (diff, undo) = generate_diff(&context.front_buffer, &context.back_buffer);

        if undo.len() > 0 && context.mode != Mode::Undo {
            context
                .undo_list
                .push((context.cursor.last_x, context.cursor.last_y, undo));
        }

        if context.mode == Mode::Undo {
            context.mode = Mode::Normal
        }

        let patch = generate_patch(diff, &context);

        context.front_buffer = context.back_buffer.clone();
        context.cursor.last_x = context.cursor.x;
        context.cursor.last_y = context.cursor.y;

        prin!("{patch}");
    }

    print!("{CLEAR_SCREEN}");

    Command::new("stty").arg("sane").status()?;

    Ok(())
}
