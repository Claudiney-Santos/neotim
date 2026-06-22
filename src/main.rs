use std::{
    env,
    process::{Command, exit},
};

use ti::{
    bindings::*,
    screen::{Context, ScreenBuffer},
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

    while process_input(&mut context)? {
        let diff = generate_diff(&context.front_buffer, &context.back_buffer);

        let patch = generate_patch(diff, &context);

        context.front_buffer = context.back_buffer.clone();

        prin!("{patch}");
    }

    print!("{CLEAR_SCREEN}");

    Command::new("stty").arg("sane").status()?;

    Ok(())
}
