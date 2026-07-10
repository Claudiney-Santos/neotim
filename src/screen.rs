use crate::{
    cursor::Cursor,
    error::{TiError, TiResult},
};
use std::{cmp::max, env, fs, process::exit};

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
    pub line_count: usize,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![Cell { char: ' ' }; width * height];

        Self {
            width,
            height,
            cells,
            line_count: 0,
        }
    }

    pub fn from(content: &Vec<u8>, width: usize, height: usize) -> Self {
        let mut cells = vec![Cell { char: ' ' }; width * height];
        let mut line_count = 0;

        let (mut x, mut y) = (0, 0);

        for c in content.iter() {
            if y >= height {
                break;
            }

            if *c as char == '\n' {
                y += 1;
                x = 0;
                line_count += 1;
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
            line_count,
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
    pub front_buffer: Vec<Cell>,
    pub back_buffer: ScreenBuffer,
    pub file_path: String,
    pub cursor: Cursor,
    pub prev_cursor: Cursor,
    pub mode: Mode,
    pub undo_list: Vec<(Cursor, Vec<(usize, usize, char)>)>,
}

impl Context {
    pub fn new() -> TiResult<Self> {
        let path = env::args()
            .nth(1)
            .ok_or_else(|| TiError("You need to provide the file path!".to_owned()))?;
        let content = fs::read(path.to_owned()).expect("File not found!");
        let (width, height) = terminal_size();

        Ok(Self {
            front_buffer: vec![Cell { char: ' ' }; width * height],
            back_buffer: ScreenBuffer::from(&content, width, height),
            cursor: Cursor::new(),
            prev_cursor: Cursor::new(),
            mode: Mode::Undo,
            file_path: path.to_owned(),
            undo_list: vec![],
        })
    }

    pub fn sync_screen_buffers(&mut self) -> Vec<(usize, usize, char)> {
        let mut diff = Vec::new();
        let mut undo = Vec::new();

        for i in 0..self.front_buffer.len() {
            if self.front_buffer[i] != self.back_buffer.cells[i] {
                let x = i % self.back_buffer.width;
                let y = i / self.back_buffer.width;
                diff.push((x, y, self.back_buffer.cells[i].char));
                undo.push((x, y, self.front_buffer[i].char));
            }
        }

        if undo.len() > 0 && self.mode != Mode::Undo {
            self.undo_list.push((self.prev_cursor, undo));
        }

        if self.mode == Mode::Undo {
            self.mode = Mode::Normal
        }

        self.front_buffer = self.back_buffer.cells.clone();
        self.prev_cursor = self.cursor;

        diff
    }
}

pub fn move_block_horizontally(
    screen: &mut ScreenBuffer,
    x: usize,
    y: usize,
    size: usize,
    steps: isize,
) -> TiResult<()> {
    if (steps < 0 && x as isize + steps < 0)
        && (steps >= 0 && x + size + steps as usize >= screen.width)
    {
        return Err(TiError("There is no space to do that action".to_owned()));
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
) -> TiResult<()> {
    if (steps < 0 && line as isize + steps < 0)
        || (steps >= 0 && (line + size) as isize >= screen.height as isize - steps)
    {
        return Err(TiError("There is no space to do that action".to_owned()));
    }

    let start = line * screen.width;
    let end = (line + size) * screen.width;

    let dest = max(0, start as isize + (screen.width as isize * steps)) as usize;

    screen.cells.copy_within(start..(end - 1), dest);

    if steps < 0 {
        for i in max(0, end as isize + steps * screen.width as isize) as usize..end {
            screen.cells[i] = Cell { char: ' ' };
        }
        screen.line_count -= steps.abs() as usize;
    } else {
        for i in start..((line + max(0, steps as usize)) * screen.width) {
            screen.cells[i] = Cell { char: ' ' };
        }
        screen.line_count += steps as usize;
    }

    Ok(())
}

pub fn backspace(screen: &mut ScreenBuffer, cursor: &mut Cursor) -> TiResult<()> {
    if cursor.x == 0 && cursor.y == 0 {
        return Ok(());
    }

    if cursor.x > 0 {
        move_block_horizontally(
            screen,
            cursor.x,
            cursor.y,
            screen.line_len(cursor.y) - cursor.x,
            -1,
        )?;
        cursor.x -= 1;
        return Ok(());
    }

    cursor.y -= 1;
    cursor.go_to_line_end(screen, Mode::Insert);

    let start = (cursor.y + 1) * screen.width;
    let end = (cursor.y + 1) * screen.width + screen.line_len(cursor.y + 1);

    let dest = cursor.y * screen.width + screen.line_len(cursor.y);

    screen.cells.copy_within(start..end, dest);

    for i in start..end {
        screen.cells[i] = Cell { char: ' ' };
    }

    if cursor.y < screen.line_count {
        move_block_vertically(screen, cursor.y + 2, screen.line_count - (cursor.y + 1), -1)?;
    }

    Ok(())
}
pub fn break_line(screen: &mut ScreenBuffer, x: usize, y: usize) -> TiResult<()> {
    if y < screen.line_count {
        move_block_vertically(screen, y + 1, screen.line_count - y, 1)?;
    }

    let start = y * screen.width + x;
    let end = y * screen.width + screen.line_len(y);

    screen.cells.copy_within(start..end, (y + 1) * screen.width);

    for i in start..end {
        screen.cells[i] = Cell { char: ' ' };
    }

    Ok(())
}
