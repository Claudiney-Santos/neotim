use super::*;
use std::{fs, process::exit};

// pub fn save(&self) {
//     fs::write(&self.path, self.content.concat().replace("\r", ""))
//         .expect("Failed to write to file");
// }

#[repr(C)]
struct WinSize {
    row: u16,
    col: u16,
    xpixel: u16,
    ypixel: u16,
}

unsafe extern "C" {
    fn ioctl(fd: i32, request: usize, out: *mut WinSize) -> i32;
}

fn terminal_size() -> (usize, usize) {
    let mut size = WinSize {
        row: 0,
        col: 0,
        xpixel: 0,
        ypixel: 0,
    };

    if unsafe { ioctl(1, 0x5413, &mut size) } != 0 {
        eprintln!("Failed to get terminal size!");
        exit(1);
    }

    (size.col as usize, size.row as usize)
}

#[derive(Clone, PartialEq)]
pub struct Cell {
    pub char: char,
}

#[derive(Clone)]
pub struct ScreenBuffer {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

impl ScreenBuffer {
    pub fn new(path: &str, width: usize, height: usize) -> Self {
        let mut cells: Vec<Cell> = Vec::with_capacity(width * height);

        cells.extend(
            fs::read(path)
                .expect("File not found!")
                .into_iter()
                .map(|char| Cell { char: char as char })
                .collect::<Vec<Cell>>(),
        );

        Self {
            width,
            height,
            cells,
        }
    }

    pub fn print(&self) {
        for cell in self.cells.iter() {
            if cell.char == '\n' {
                print!("\r");
            }

            print!("{}", cell.char);
        }
    }
}

pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub block: bool,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            block: true,
        }
    }
    pub fn build(&self) -> String {
        let mut building = String::new();

        building.push_str(&format!("\x1b[{};{}H", self.y + 1, self.x + 1));

        if self.block {
            building.push_str(&format!("\x1b[2 q"));
        } else {
            building.push_str(&format!("\x1b[6 q"));
        }

        building
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    // VISUAL,
    Insert,
}

pub struct Context {
    pub front_buffer: ScreenBuffer,
    pub back_buffer: ScreenBuffer,
    pub cursor: Cursor,
    pub mode: Mode,
}

impl Context {
    pub fn new(path: &str) -> Self {
        let (width, height) = terminal_size();
        let screen_buffer = ScreenBuffer::new(path, width, height);

        Self {
            front_buffer: screen_buffer.clone(),
            back_buffer: screen_buffer,
            cursor: Cursor::new(),
            mode: Mode::Normal,
        }
    }
}
