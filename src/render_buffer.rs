// TODO: create the render_buffer
// Its reponsabity is result on the final string that modifies the terminal content
use crate::{
    HIDE_CURSOR, SHOW_CURSOR, document::Document, terminal::get_terminal_size, viewport::Viewport,
};

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

pub struct Pos {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone)]
pub struct RenderBuffer {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

impl RenderBuffer {
    // TODO: RenderBuffer and Viewport has a info leaked: the terminal size
    pub fn new() -> Self {
        let (width, height) = get_terminal_size();

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
        }
    }

    pub fn from(doc: &Document, viewport: &Viewport) -> Self {
        let mut render_buffer = Self::new();

        for (row, line) in doc.get_content().iter().skip(viewport.top_row).enumerate() {
            for (col, ch) in line
                .chars()
                .skip(viewport.left_column)
                .map(|ch| if ch != ' ' { ch } else { '·' })
                .enumerate()
            {
                render_buffer.cells[row * viewport.width + col].char = ch;
            }
        }

        render_buffer
    }

    pub fn diff(&self, old: &Self) -> Vec<(Pos, Cell)> {
        let mut diff = vec![];

        for (i, cell) in self.cells.iter().enumerate() {
            if &old.cells[i] != cell {
                let col = i % self.width;
                let row = i / self.width;
                diff.push((Pos { row, col }, *cell));
            }
        }

        diff
    }

    pub fn patch(diff: Vec<(Pos, Cell)>) -> String {
        let mut render = String::new();

        render.push_str(HIDE_CURSOR);

        for (pos, cell) in diff {
            let gray = if cell.char == '·' { "90" } else { "0" };
            let highlighted = if cell.highlight { ";40" } else { "" };

            render.push_str(&format!(
                "\x1b[{};{}H\x1b[{}{}m{}\x1b[0m",
                pos.row + 1,
                pos.col + 1,
                gray,
                highlighted,
                cell.char,
            ));
        }

        render.push_str(SHOW_CURSOR);

        render
    }
}
