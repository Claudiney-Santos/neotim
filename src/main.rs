use std::{
    env,
    io::{Read, stdin},
    process::{Command, exit},
};

use ti::{
    screen::{Context, Cursor, ScreenBuffer},
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

fn generate_patch(diff: Vec<(usize, usize, char)>, cursor: &Cursor) -> String {
    let mut render = String::new();

    render.push_str(HIDE_CURSOR);

    for (cx, cy, char) in diff {
        render.push_str(&format!("\x1b[{};{}H{}", cy + 1, cx + 1, char));
    }

    render.push_str(&cursor.build());

    render.push_str(SHOW_CURSOR);

    render
}

fn process_input(context: &mut Context) -> anyhow::Result<bool> {
    let mut key = [0; 1];
    stdin().read_exact(&mut key)?;

    match key[0] as char {
        'q' => return Ok(false),
        'h' if context.cursor.x as i32 > 0 => context.cursor.x -= 1,
        'j' => context.cursor.y += 1,
        'k' if context.cursor.y as i32 > 0 => context.cursor.y -= 1,
        'l' => context.cursor.x += 1,
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
    prin!("{}", context.cursor.build());

    loop {
        if !process_input(&mut context)? {
            break;
        }

        let diff = generate_diff(&context.front_buffer, &context.back_buffer);

        let patch = generate_patch(diff, &context.cursor);

        context.front_buffer = context.back_buffer.clone();

        prin!("{patch}");
    }

    print!("{CLEAR_SCREEN}");

    Command::new("stty").arg("sane").status()?;

    Ok(())
}
