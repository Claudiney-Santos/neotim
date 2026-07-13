use std::{panic, process::Command};
use ti::{
    bindings::*,
    screen::{Cell, Context},
    *,
};

pub fn generate_patch(diff: Vec<(usize, usize, Cell)>) -> String {
    let mut render = String::new();

    render.push_str(HIDE_CURSOR);

    for (cx, cy, cell) in diff {
        let gray = if cell.char == '·' { "90" } else { "0" };
        let highlighted = if cell.highlight { ";40" } else { "" };

        render.push_str(&format!(
            "\x1b[{};{}H\x1b[{}{}m{}\x1b[0m",
            cy + 1,
            cx + 1,
            gray,
            highlighted,
            cell.char,
        ));
    }

    render.push_str(SHOW_CURSOR);

    render
}

fn main() -> anyhow::Result<()> {
    let mut context = Context::new()?;

    panic::set_hook(Box::new(|panic_info| {
        print!("{CLEAR_SCREEN}\x1b[2 q");
        Command::new("stty").arg("sane").status().unwrap();

        println!("🚨 Fuck! Some shit happened.");

        if let Some(location) = panic_info.location() {
            println!(
                "On this file: '{}', line: {}",
                location.file(),
                location.line()
            );
        }

        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            println!("Panic message: {s}");
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            println!("Panic message: {s}");
        }
    }));

    Command::new("stty").args(["raw", "-echo"]).status()?;
    print!("{CLEAR_SCREEN}");

    loop {
        let diff = context.sync_screen_buffers();

        prin!(
            "{}{}",
            generate_patch(diff),
            context.cursor.build(&context.back_buffer, context.mode)
        );

        if !process_input(&mut context).unwrap() {
            break;
        }
    }

    print!("{CLEAR_SCREEN}");
    Command::new("stty").arg("sane").status()?;

    Ok(())
}
