use super::*;
use std::fs;

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    // VISUAL,
    Insert,
}

pub struct Ctx {
    pub mode: Mode,
    pub content: Vec<String>,
    pub path: String,
    pub x: usize,
    pub y: usize,
}

impl Ctx {
    pub fn new(path: &str) -> Result<Self, anyhow::Error> {
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

    pub fn up(&mut self, n: usize) {
        if self.y as i32 - n as i32 >= 0 {
            self.y -= n;
            prin!("\x1b[{n}A")
        }
    }

    pub fn down(&mut self, n: usize) {
        if self.y + n < self.content.len() {
            self.y += n;
            prin!("\x1b[{n}B")
        }
    }

    pub fn right(&mut self, n: usize) {
        self.x += n;
        prin!("\x1b[{n}C")
    }

    pub fn left(&mut self, n: usize) {
        if self.x as i32 - n as i32 >= 0 {
            self.x -= n;
            prin!("\x1b[{n}D")
        }
    }

    pub fn go_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
        prin!("\x1b[{y};{x}H");
    }

    pub fn thin_cursor(&self) {
        prin!("\x1b[6 q");
    }

    pub fn block_cursor(&self) {
        prin!("\x1b[2 q");
    }

    pub fn print_all(&self) {
        for line in self.content.iter() {
            prin!("{line}");
        }
    }

    pub fn save(&self) {
        fs::write(&self.path, self.content.concat().replace("\r", ""))
            .expect("Failed to write to file");
    }
}
