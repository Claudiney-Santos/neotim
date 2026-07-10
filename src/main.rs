use std::{env, process::Command};
use ti::{bindings::*, screen::Context, *};

pub fn generate_patch(diff: Vec<(usize, usize, char)>) -> String {
    let mut render = String::new();

    render.push_str(HIDE_CURSOR);

    for (cx, cy, char) in diff {
        render.push_str(&format!(
            "\x1b[{};{}H\x1b[{}m{}\x1b[0m",
            cy + 1,
            cx + 1,
            if char == '·' { "90" } else { "0" },
            char,
        ));
    }

    render.push_str(SHOW_CURSOR);

    render
}

fn main() -> anyhow::Result<()> {
    let mut context = Context::new()?;

    Command::new("stty").args(["raw", "-echo"]).status()?;
    print!("{CLEAR_SCREEN}");

    loop {
        let diff = context.sync_screen_buffers();

        prin!(
            "{}{}",
            generate_patch(diff),
            context.cursor.build(&context.back_buffer, context.mode)
        );

        if !process_input(&mut context)? {
            break;
        }
    }

    print!("{CLEAR_SCREEN}");
    Command::new("stty").arg("sane").status()?;

    Ok(())
}
