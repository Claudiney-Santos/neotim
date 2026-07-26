// TODO: create the render_buffer
// Its reponsabity is result on the final string that modifies the terminal content
use crate::{
    HIDE_CURSOR, SHOW_CURSOR,
    app::{App, Mode},
    terminal::get_terminal_size,
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

    pub fn from(
        App {
            doc,
            viewport,
            mode,
            cursor,
            ..
        }: &App,
    ) -> Self {
        let mut render_buffer = Self::new();

        for (row, line) in doc
            .get_content()
            .iter()
            .skip(viewport.top_row)
            .take(viewport.height)
            .enumerate()
        {
            for (col, char) in line
                .chars()
                .skip(viewport.left_column)
                .map(|ch| if ch != ' ' { ch } else { '·' })
                .enumerate()
            {
                let highlight = match mode {
                    Mode::Visual(landmark) => {
                        let (mut start, mut end) = if landmark.row < cursor.row
                            || (landmark.row == cursor.row && landmark.col <= cursor.col)
                        {
                            (*landmark, cursor.to_pos())
                        } else {
                            (cursor.to_pos(), *landmark)
                        };

                        start.row -= viewport.top_row;
                        end.row -= viewport.top_row;

                        (start.row < row || (start.row == row && start.col <= col))
                            && (row < end.row || (row == end.row && col <= end.col))
                    }
                    _ => false,
                };

                render_buffer.cells[row * viewport.width + col] = Cell { char, highlight };
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
