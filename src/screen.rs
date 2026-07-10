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

pub fn move_block_horizontally(
    screen: &mut ScreenBuffer,
    x: usize,
    y: usize,
    size: usize,
    steps: isize,
) -> Result<(), String> {
    if (steps < 0 && x as isize + steps < 0)
        && (steps >= 0 && x + size + steps as usize >= screen.width)
    {
        return Err(String::from("There is no space to do that action"));
    }

    let start = y * screen.width + x;
    let end = y * screen.width + x + size;

    screen
        .cells
        .copy_within(start..end, max(0, start as isize + steps) as usize);

    if steps >= 0 {
        for i in start..start + steps as usize {
            screen.cells[i] = Cell { char: '·' };
        }
    } else {
        for i in max(0, end as isize + steps) as usize..end {
            screen.cells[i] = Cell { char: ' ' };
        }
    }

    Ok(())
}

pub fn move_block_vertically(
    screen: &mut ScreenBuffer,
    line: usize,
    size: usize,
    steps: isize,
) -> Result<(), String> {
    if (steps < 0 && line as isize + steps < 0)
        || (steps >= 0 && (line + size) as isize >= screen.height as isize - steps)
    {
        return Err(String::from("There is no space to do that action"));
    }

    let start = line * screen.width;
    let end = (line + size) * screen.width;

    let dest = max(0, start as isize + (screen.width as isize * steps)) as usize;

    screen.cells.copy_within(start..(end - 1), dest);

    if steps < 0 {
        for i in max(0, end as isize + steps * screen.width as isize) as usize..end {
            screen.cells[i] = Cell { char: ' ' };
        }
    } else {
        for i in start..((line + max(0, steps as usize)) * screen.width) {
            screen.cells[i] = Cell { char: ' ' };
        }
    }

    Ok(())
}

pub fn backspace(screen: &mut ScreenBuffer, cursor: &mut Cursor) -> Result<(), String> {
    if cursor.x == 0 && cursor.y == 0 {
        return Ok(());
    }

    if cursor.x > 0 {
        move_block_horizontally(
            screen,
            cursor.x,
            cursor.y,
            screen.last_char(cursor.y) - cursor.x,
            -1,
        )?;
        cursor.x -= 1;
        return Ok(());
    }

    let start = cursor.y * screen.width;
    let end = cursor.y * screen.width + screen.last_char(cursor.y);

    let dest = (cursor.y - 1) * screen.width + screen.last_char(cursor.y - 1);

    screen.cells.copy_within(start..end, dest);

    for i in start..end {
        screen.cells[i] = Cell { char: ' ' };
    }

    move_block_vertically(screen, cursor.y + 1, screen.last_line() - cursor.y - 1, -1)?;

    cursor.x = screen.last_char(cursor.y - 1);
    cursor.y -= 1;

    Ok(())
}
pub fn break_line(screen: &mut ScreenBuffer, x: usize, y: usize) -> Result<(), String> {
    move_block_vertically(screen, y + 1, screen.last_line() + 1 - y, 1)?;

    let start = y * screen.width + x;
    let end = y * screen.width + screen.line_len(y);

    screen.cells.copy_within(start..end, (y + 1) * screen.width);

    for i in start..end {
        screen.cells[i] = Cell { char: ' ' };
    }

    Ok(())
}
