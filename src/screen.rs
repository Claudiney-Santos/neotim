use std::{cmp::min, fs, process::exit};

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

    pub fn last_char(&self, line: usize) -> usize {
        let mut counter = 0;
        for i in self.width * line..self.width * (line + 1) {
            if self.cells[i].char == ' ' {
                break;
            }
            counter += 1;
        }

        counter
    }

    pub fn last_line(&self) -> usize {
        let mut counter = 0;
        while counter < self.height {
            let line = counter * self.width;
            if self.cells[line].char == ' ' {
                break;
            }
            counter += 1;
        }

        counter
    }
}

#[derive(Copy, Clone)]
pub enum CursorMode {
    Block = 2,
    Underline = 4,
    Bar = 6,
}

pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub mode: CursorMode,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            mode: CursorMode::Block,
        }
    }
    pub fn build(&self, limit: Option<usize>) -> String {
        let mut building = String::new();

        let x = if let Some(l) = limit {
            min(self.x, l)
        } else {
            self.x
        };

        building.push_str(&format!("\x1b[{};{}H", self.y + 1, x + 1));

        building.push_str(&format!("\x1b[{} q", self.mode as usize));

        building
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    Replace,
    Delete,
    // VISUAL,
    Insert,
}

pub struct Context {
    pub front_buffer: ScreenBuffer,
    pub back_buffer: ScreenBuffer,
    pub file_path: String,
    pub cursor: Cursor,
    pub mode: Mode,
    pub lines: Vec<usize>,
}

impl Context {
    pub fn new(path: &str) -> Self {
        let content = fs::read(path).expect("File not found!");
        let (width, height) = terminal_size();

        let screen_buffer = ScreenBuffer::new(&content, width, height);

        let mut lines: Vec<usize> = Vec::new();
        let mut counter = 0;
        for c in content.iter() {
            if *c as char != '\n' {
                counter += 1;
                continue;
            }

            lines.push(counter);
            counter = 0;
        }

        Self {
            front_buffer: screen_buffer.clone(),
            back_buffer: screen_buffer,
            cursor: Cursor::new(),
            mode: Mode::Normal,
            file_path: path.to_owned(),
            lines,
        }
    }
}
