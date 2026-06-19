use std::{
    fs,
    io::{Read, stdin},
    process::{Command, exit},
};

macro_rules! prin {
    ($($arg:tt)*) => {{
        print!($($arg)*);
        std::io::Write::flush(&mut std::io::stdout()).expect("Error on flush prin!");
    }};
}

const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";

#[derive(PartialEq)]
enum Mode {
    Normal,
    // VISUAL,
    Insert,
}

struct Ctx {
    mode: Mode,
    content: Vec<String>,
    x: usize,
    y: usize,
}

impl Ctx {
    fn new() -> Result<Self, anyhow::Error> {
        let content = fs::read_to_string("./README.md")?
            .split("\n")
            .map(|l| format!("{l}\r\n"))
            .collect::<Vec<String>>();

        Ok(Self {
            mode: Mode::Normal,
            content,
            x: 0,
            y: 0,
        })
    }

    fn up(&mut self, n: usize) {
        if self.y as i32 - n as i32 >= 0 {
            self.y -= n;
            prin!("\x1b[{n}A")
        }
    }

    fn down(&mut self, n: usize) {
        if self.y + n < self.content.len() {
            self.y += n;
            prin!("\x1b[{n}B")
        }
    }

    fn right(&mut self, n: usize) {
        self.x += n;
        prin!("\x1b[{n}C")
    }

    fn left(&mut self, n: usize) {
        if self.x as i32 - n as i32 >= 0 {
            self.x -= n;
            prin!("\x1b[{n}D")
        }
    }

    fn go_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
        prin!("\x1b[{y};{x}H");
    }

    fn thin_cursor(&self) {
        prin!("\x1b[6 q");
    }

    fn block_cursor(&self) {
        prin!("\x1b[2 q");
    }

    fn print_all(&self) {
        for line in self.content.iter() {
            prin!("{line}");
        }
    }
}

fn exec_command(_: &Ctx) -> Result<(), anyhow::Error> {
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
        // #TODO: Save content
    }

    if result.contains("q") {
        Command::new("stty").arg("sane").status()?;
        print!("{CLEAR_SCREEN}");
        exit(0);
    }

    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let mut ctx = Ctx::new()?;

    Command::new("stty").args(["raw", "-echo"]).status()?;

    print!("{CLEAR_SCREEN}");
    ctx.print_all();
    ctx.go_to(0, 0);

    loop {
        let mut key = [0; 1];
        stdin().read_exact(&mut key)?;

        if ctx.mode == Mode::Normal {
            match key[0] as char {
                'q' => break,
                'h' => ctx.left(1),
                'j' => ctx.down(1),
                'k' => ctx.up(1),
                'l' => ctx.right(1),
                'i' => {
                    ctx.mode = Mode::Insert;
                    ctx.thin_cursor();
                }
                ':' => exec_command(&ctx)?,
                _ => {}
            }
        } else {
            match key[0] as char {
                '\x1b' => {
                    ctx.mode = Mode::Normal;
                    ctx.block_cursor();
                }
                c => {
                    prin!("\x1b[1@{}", c); // Write aside right letters
                    ctx.content[ctx.y].insert(ctx.x, c);
                    ctx.x += 1;
                }
            }
        }
    }

    Command::new("stty").arg("sane").status()?;
    print!("{CLEAR_SCREEN}");

    Ok(())
}

// prin!("\x1b[1L"); break line
