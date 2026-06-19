use std::{
    env, fs,
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

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Normal,
    // VISUAL,
    Insert,
}

struct Ctx {
    mode: Mode,
    content: Vec<String>,
    path: String,
    x: usize,
    y: usize,
}

impl Ctx {
    fn new(path: &str) -> Result<Self, anyhow::Error> {
        let content = fs::read_to_string(path)?
            .split("\n")
            .map(|l| format!("{l}\r\n"))
            .collect::<Vec<String>>();

        Ok(Self {
            mode: Mode::Normal,
            content,
            path: path.to_owned(),
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

    fn save(&self) {
        fs::write(&self.path, self.content.concat().replace("\r", ""))
            .expect("Failed to write to file");
    }
}

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
        panic!("You need to provide the file path!");
    }

    let mut ctx = Ctx::new(&args[1])?;

    Command::new("stty").args(["raw", "-echo"]).status()?;

    // print!("{CLEAR_SCREEN}");
    ctx.print_all();
    ctx.go_to(0, 0);

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
            (Mode::Insert, c) => {
                prin!("\x1b[1@{}", c); // Write aside right letters
                ctx.content[ctx.y].insert(ctx.x, c);
                ctx.x += 1;
            }
            _ => {}
        }
    }

    Command::new("stty").arg("sane").status()?;
    // print!("{CLEAR_SCREEN}");

    Ok(())
}

// prin!("\x1b[1L"); break line
