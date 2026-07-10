use crate::cursor::Cursor;
use std::{
    cmp::{max, min},
    fs,
    process::exit,
};

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

#[derive(Clone, PartialEq, Copy, Debug)]
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
    pub fn new(content: &Vec<u8>, width: usize, height: usize) -> Self {
        let mut cells = vec![Cell { char: ' ' }; width * height];

        let (mut x, mut y) = (0, 0);

        for c in content.iter() {
            if y >= height {
                break;
            }

            if *c as char == '\n' {
                y += 1;
                x = 0;
                continue;
            }

            if x >= width {
                y += 1;
                x = 0;
                if y >= height {
                    break;
                }
            }

            if *c as char == ' ' {
                cells[y * width + x] = Cell { char: '·' };
                x += 1;
                continue;
            }

            cells[y * width + x] = Cell { char: *c as char };
            x += 1;
        }

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

            if cell.char == '·' {
                print!("\x1b[90m·\x1b[0m");
                continue;
            }

            print!("{}", cell.char);
        }
    }

    pub fn line_len(&self, line: usize) -> usize {
        let mut counter = 0;
        for i in self.width * line..self.width * (line + 1) {
            if self.cells[i].char == ' ' {
                break;
            }
            counter += 1;
        }

        counter
    }

    pub fn last_char(&self, line: usize) -> usize {
        let mut counter = 0;
        for i in self.width * line..self.width * (line + 1) {
            if self.cells[i].char == ' ' {
                break;
            }
            counter += 1;
        }

        max(counter as isize - 1, 0) as usize
    }

    pub fn last_line(&self) -> usize {
        let mut counter = self.width * self.height - 1;
        while counter > 0 {
            if self.cells[counter].char != ' ' {
                break;
            }
            counter -= 1;
        }

        max((counter / self.width) as isize - 1, 0) as usize
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    Replace,
    Delete,
    Insert,
    Undo,
}

pub struct Context {
    pub front_buffer: ScreenBuffer,
    pub back_buffer: ScreenBuffer,
    pub file_path: String,
    pub cursor: Cursor,
    pub mode: Mode,
    pub undo_list: Vec<(usize, usize, Vec<(usize, usize, char)>)>,
}

impl Context {
    pub fn new(path: &str) -> Self {
        let content = fs::read(path).expect("File not found!");
        let (width, height) = terminal_size();

        let screen_buffer = ScreenBuffer::new(&content, width, height);

        Self {
            front_buffer: screen_buffer.clone(),
            back_buffer: screen_buffer,
            cursor: Cursor::new(),
            mode: Mode::Normal,
            file_path: path.to_owned(),
            undo_list: vec![],
        }
    }

    pub fn get_min_x(&self) -> usize {
        min(self.back_buffer.last_char(self.cursor.y), self.cursor.x)
    }

    pub fn get_min_y(&self) -> usize {
        min(self.back_buffer.last_line(), self.back_buffer.height)
    }
}
