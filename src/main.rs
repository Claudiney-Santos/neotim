use std::{
    fs,
    io::{Read, stdin},
    process::Command,
};

macro_rules! prin {
    ($($arg:tt)*) => {{
        print!($($arg)*);
        std::io::Write::flush(&mut std::io::stdout()).expect("Error on flush prin!");
    }};
}

const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";
// const HIDE_CURSOR: &str = "\x1b[?25l";
// const SHOW_CURSOR: &str = "\x1b[?25h";
// const UP: &str = "\x1b[1A";
// const DOWN: &str = "\x1b[1B";
// const RIGHT: &str = "\x1b[1C";
// const LEFT: &str = "\x1b[1D";

// enum Mode {
//     NORMAL,
//     VISUAL,
//     INSERT,
// }

struct Ctx {
    // mode: Mode,
    content: String,
    size: i32,
    x: i32,
    y: i32,
}

impl Ctx {
    fn new() -> Result<Self, anyhow::Error> {
        let content = fs::read_to_string("./README.md")?.replace("\n", "\r\n");
        let size = content.matches("\n").count() as i32;

        Ok(Self {
            content,
            size,
            x: 0,
            y: 0,
        })
    }

    fn up(&mut self, n: i32) {
        self.y -= n;
        prin!("\x1b[{n}A")
    }

    fn down(&mut self, n: i32) {
        if self.y + n < self.size {
            self.y += n;
            prin!("\x1b[{n}B")
        }
    }

    fn right(&mut self, n: i32) {
        self.x += n;
        prin!("\x1b[{n}C")
    }

    fn left(&mut self, n: i32) {
        self.x -= n;
        prin!("\x1b[{n}D")
    }

    fn go_to(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
        prin!("\x1b[{y};{x}H");
    }
}

fn main() -> Result<(), anyhow::Error> {
    let mut ctx = Ctx::new()?;

    Command::new("stty").args(["raw", "-echo"]).status()?;

    print!("{CLEAR_SCREEN}");
    print!("{}", ctx.content);
    prin!("{}", ctx.size);

    ctx.go_to(0, 0);
    loop {
        let mut key = [0; 1];
        stdin().read_exact(&mut key)?;

        match key[0] as char {
            'q' => break,
            'h' => ctx.left(1),
            'j' => ctx.down(1),
            'k' => ctx.up(1),
            'l' => ctx.right(1),
            _ => {}
        }
    }

    Command::new("stty").arg("sane").status()?;

    Ok(())
}
