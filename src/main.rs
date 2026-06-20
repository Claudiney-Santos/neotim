use std::{
    env,
    io::{Read, stdin},
    process::{Command, exit},
};

use ti::{context::*, *};

fn exec_command(ctx: &Ctx) -> Result<bool, anyhow::Error> {
    let mut result = String::from(":");

    loop {
        let mut key = [0; 1];
        stdin().read_exact(&mut key)?;

        match key[0] as char {
            '\r' | '\n' => break,
            c => result.push(c),
        }
    }

    if result.contains("w") {
        ctx.save();
    }

    if result.contains("q") {
        return Ok(true);
    }

    Ok(false)
}

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("You need to provide the file path!");
        exit(1);
    }

    let mut ctx = Ctx::new(&args[1])?;

    Command::new("stty").args(["raw", "-echo"]).status()?;

    print!("{CLEAR_SCREEN}");
    ctx.print_all();
    prin!("{CURSOR_HOME}");

    loop {
        let mut key = [0; 1];
        stdin().read_exact(&mut key)?;

        match (ctx.mode, key[0] as char) {
            (Mode::Normal, 'h') => ctx.left(1),
            (Mode::Normal, 'j') => ctx.down(1),
            (Mode::Normal, 'k') => ctx.up(1),
            (Mode::Normal, 'l') => ctx.right(1),
            (Mode::Normal, 'i') => {
                ctx.mode = Mode::Insert;
                ctx.thin_cursor();
            }
            (Mode::Normal, ':') => {
                if exec_command(&ctx)? {
                    break;
                }
            }
            (Mode::Insert, '\x1b') => {
                ctx.mode = Mode::Normal;
                ctx.block_cursor();
            }
            (Mode::Insert, '\x7f') => {
                if ctx.x as i32 > 0 {
                    ctx.content[ctx.y].remove(ctx.x);
                    let rest = &ctx.content[ctx.y][ctx.x..ctx.content[ctx.y].len() - 2];
                    prin!("\x08");
                    prin!("\x1b[K");
                    prin!("{rest}");
                    let rest_size = rest.chars().count();
                    if rest_size > 0 {
                        prin!("\x1b[{}D", rest_size); // Move o cursor 'n' vezes para a esquerda
                    }
                    ctx.x -= 1;
                }
            }
            (Mode::Insert, '\n' | '\r') => {
                if ctx.x == 0 {
                    // prin!("\x1b[K");
                    prin!("\r\n");
                    // prin!("\x1b[1L");
                    ctx.content.insert(ctx.y, String::from("\r\n"));
                    ctx.y += 1;
                    print!("{}", ctx.content[ctx.y]);
                } else {
                    prin!("\x1b[K");
                    pub const CLEAR_TO_START: &str = "\x1b[1K"; // Clears from cursor to start of line
                    prin!("{CLEAR_TO_END}");
                    prin!("{BREAK_LINE}");
                    prin!("\x1b[1L");
                    let after = ctx.content[ctx.y].split_off(ctx.x);
                    ctx.x = 0;
                    ctx.y += 1;
                    print!("{after}");
                    ctx.content.insert(ctx.y, after);
                    ctx.up(1);
                    // prin!("\x1b[1S"); // Empurra todo o texto 1 linha para cima
                    // break line;
                }
            }
            (Mode::Insert, c) => {
                prin!("\x1b[1@{}", c); // Write aside right letters
                ctx.content[ctx.y].insert(ctx.x, c);
                ctx.x += 1;
            }
            _ => {}
        }
    }

    Command::new("stty").arg("sane").status()?;
    print!("{CLEAR_SCREEN}");
    ctx.print_all();

    Ok(())
}

// prin!("\x1b[1L"); break line
