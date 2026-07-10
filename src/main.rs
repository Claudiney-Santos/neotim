use std::{
    env,
    process::{Command, exit},
};

use ti::{
    bindings::*,
    screen::{Context, Mode, generate_diff},
    *,
};

pub fn generate_patch(diff: Vec<(usize, usize, char)>) -> String {
    let mut render = String::new();

    render.push_str(HIDE_CURSOR);

    for (cx, cy, char) in diff {
        if char == '·' {
            render.push_str(&format!("\x1b[{};{}H\x1b[90m·\x1b[0m", cy + 1, cx + 1));
            continue;
        }

        render.push_str(&format!("\x1b[{};{}H{}", cy + 1, cx + 1, char));
    }

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

    loop {
        let (diff, undo) = generate_diff(&context.front_buffer, &context.back_buffer);

        if undo.len() > 0 && context.mode != Mode::Undo {
            context
                .undo_list
                .push((context.cursor.last_x, context.cursor.last_y, undo));
        }

        if context.mode == Mode::Undo {
            context.mode = Mode::Normal
        }

        context.front_buffer = context.back_buffer.clone();
        context.cursor.last_x = context.cursor.x;
        context.cursor.last_y = context.cursor.y;

        prin!(
            "{}{}",
            generate_patch(diff),
            &context.cursor.build(&context.back_buffer, context.mode)
        );

        if !process_input(&mut context)? {
            break;
        }
    }

    print!("{CLEAR_SCREEN}");
    Command::new("stty").arg("sane").status()?;

    Ok(())
}
