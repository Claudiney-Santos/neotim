use crate::{
    cursor::{Cursor, x_bounded},
    error::{TiError, TiResult},
    undo::{UndoEntry, UndoStack},
};
use std::{
    cmp::{max, min},
    env, fs,
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
    pub highlight: bool,
}

impl Cell {
    pub fn new(char: char) -> Self {
        Self {
            char,
            highlight: false,
        }
    }
}

#[derive(Clone)]
pub struct ScreenBuffer {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
    pub line_count: usize,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize, line_count: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![
                Cell {
                    char: ' ',
                    highlight: false
                };
                width * height
            ],
            line_count,
        }
    }

    pub fn from(content: &Vec<u8>, width: usize, height: usize) -> Self {
        let mut cells = vec![
            Cell {
                char: ' ',
                highlight: false
            };
            width * height
        ];
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
                cells[y * width + x] = Cell::new('·');
                x += 1;
                continue;
            }

            cells[y * width + x] = Cell {
                char: *c as char,
                highlight: false,
            };
            x += 1;
        }

        Self {
            width,
            height,
            cells,
            line_count,
        }
    }

    pub fn set_highlighted_cells(&mut self, cursor: Cursor, prev_cursor: Cursor, mode: Mode) {
        if let Mode::Visual(landmark) = mode {
            let cursor_idx = cursor.y * self.width + cursor.x;
            let prev_cursor_idx = prev_cursor.y * self.width + prev_cursor.x;

            let start = min(landmark, cursor_idx);
            let end = max(landmark, cursor_idx);

            for i in start..end {
                let actual_x = i % self.width;
                let x_bounded = x_bounded(
                    (i % self.width) as isize,
                    i / self.width,
                    self,
                    Mode::Normal,
                );

                if actual_x <= x_bounded {
                    self.cells[i].highlight = true;
                }
            }

            if prev_cursor_idx < start {
                for i in prev_cursor_idx..start {
                    self.cells[i].highlight = false;
                }
            } else if prev_cursor_idx > end {
                for i in end..prev_cursor_idx {
                    self.cells[i].highlight = false;
                }
            }
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
    Visual(usize),
    Undo,
}

pub struct Context {
    pub front_buffer: ScreenBuffer,
    pub back_buffer: ScreenBuffer,
    pub file_path: String,
    pub cursor: Cursor,
    pub prev_cursor: Cursor,
    pub mode: Mode,
    pub undo_stack: UndoStack,
}

impl Context {
    pub fn new() -> TiResult<Self> {
        let path = env::args()
            .nth(1)
            .ok_or_else(|| TiError("You need to provide the file path!".to_owned()))?;
        let content = fs::read(path.to_owned()).expect("File not found!");
        let (width, height) = terminal_size();

        let back_buffer = ScreenBuffer::from(&content, width, height);

        Ok(Self {
            front_buffer: ScreenBuffer::new(width, height, back_buffer.line_count),
            back_buffer,
            cursor: Cursor::new(),
            prev_cursor: Cursor::new(),
            mode: Mode::Undo,
            file_path: path.to_owned(),
            undo_stack: UndoStack::new(),
        })
    }

    pub fn sync_screen_buffers(&mut self) -> Vec<(usize, usize, Cell)> {
        let mut diff = Vec::new();
        let mut undo_delta = Vec::new();

        self.back_buffer
            .set_highlighted_cells(self.cursor, self.prev_cursor, self.mode);

        for i in 0..self.front_buffer.cells.len() {
            if self.front_buffer.cells[i] != self.back_buffer.cells[i] {
                let x = i % self.back_buffer.width;
                let y = i / self.back_buffer.width;
                diff.push((x, y, self.back_buffer.cells[i]));
                self.front_buffer.cells[i].highlight = false;
                undo_delta.push((x, y, self.front_buffer.cells[i]));
            }
        }

        if undo_delta.len() > 0 || self.front_buffer.line_count != self.back_buffer.line_count {
            self.undo_stack.push(
                self.mode,
                UndoEntry {
                    delta: undo_delta,
                    line_count: self.front_buffer.line_count,
                    cursor: self.prev_cursor,
                },
            );
        }

        if self.mode == Mode::Undo {
            self.mode = Mode::Normal
        }

        self.front_buffer.cells = self.back_buffer.cells.clone();
        self.front_buffer.line_count = self.back_buffer.line_count;
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
            screen.cells[i] = Cell::new('·');
        }
    } else {
        for i in max(0, end as isize + steps) as usize..end {
            screen.cells[i] = Cell::new(' ');
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
    if size == 0 || steps == 0 {
        return Ok(());
    }

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
            screen.cells[i] = Cell::new(' ');
        }
        screen.line_count -= steps.abs() as usize;
    } else {
        for i in start..((line + max(0, steps as usize)) * screen.width) {
            screen.cells[i] = Cell::new(' ');
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
        screen.cells[i] = Cell::new(' ');
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
        screen.cells[i] = Cell::new(' ');
    }

    Ok(())
}

pub fn copy(screen: &mut ScreenBuffer, start: usize, end: usize) -> String {
    let mut result = String::new();

    assert!(start <= end, "start is greater than end!");
    assert!(
        start < screen.cells.len(),
        "start is greater screen matrix length!"
    );
    assert!(
        end < screen.cells.len(),
        "end is greater screen matrix length!"
    );

    for i in start..end {
        result.push(screen.cells[i].char);
    }

    result
}

pub fn cut(screen: &mut ScreenBuffer, start: usize, end: usize) -> String {
    let mut result = String::new();

    assert!(start <= end, "start is greater than end!");
    assert!(
        start < screen.cells.len(),
        "start is greater screen matrix length!"
    );
    assert!(
        end < screen.cells.len(),
        "end is greater screen matrix length!"
    );

    for i in start..end {
        result.push(screen.cells[i].char);
        screen.cells[i] = Cell::new(' ');
    }

    result
}

// This paste the String until the end of current line
pub fn paste(screen: &mut ScreenBuffer, pos: usize, content: String) {
    assert!(
        pos < screen.cells.len(),
        "end is greater screen matrix length!"
    );

    let start = pos;
    let end = min(
        pos + screen.width - (pos % screen.width),
        start + content.len(),
    );

    let content = content.chars().collect::<Vec<char>>();

    for i in start..end {
        screen.cells[i].char = content[i - start];
    }
}
