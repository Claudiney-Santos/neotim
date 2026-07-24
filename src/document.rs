// TODO: create document struct
// Its responsability is handle the reading, updating, and writing file document

use std::cmp::{max, min};
use std::{
    env, fs,
    io::{self},
};

use crate::{app::Mode, error::TiError};

pub struct Pos {
    pub row: usize,
    pub col: usize,
}

pub struct Document {
    file_path: String,
    lines: Vec<String>,
}

impl Document {
    pub fn new() -> anyhow::Result<Self> {
        let file_path = env::args()
            .nth(1)
            .ok_or_else(|| TiError("You need to provide the file path!".to_owned()))?;

        Ok(Self {
            file_path: file_path.to_owned(),
            lines: fs::read_to_string(file_path)?
                .split("\n")
                .map(|line| line.to_owned())
                .collect::<Vec<String>>(),
        })
    }

    pub fn get_content(&self) -> &Vec<String> {
        &self.lines
    }

    pub fn save(&self) -> io::Result<()> {
        fs::write(&self.file_path, self.lines.join("\n"))
    }

    pub fn insert(&mut self, col: usize, row: usize, str: &str) {
        self.lines[row].insert_str(col, str)
    }

    pub fn remove_char(&mut self, col: usize, row: usize) {
        self.lines[row].remove(col);
    }

    pub fn delete(&mut self, mut start: Pos, end: Pos) {
        assert!(
            start.row < end.row || (start.row == end.row && start.col <= end.col),
            "You messed up with start and end boundaries"
        );

        let delete_start_line = if start.col == self.lines[start.row].len() {
            start.col -= 1;
            true
        } else {
            false
        };

        let delete_end_line = end.col == self.lines[end.row].len();

        if (delete_start_line || delete_end_line) && start.row + 1 < self.lines.len() {
            return;
        }

        if start.row == end.row {
            if (delete_start_line || delete_end_line) && start.row + 1 < self.lines.len() {
                let next_line = self.lines.remove(start.row + 1);
                self.lines[start.row].push_str(&next_line);
            }

            self.lines[start.row].drain(start.col..end.col);
            return;
        }

        self.lines[start.row].drain(start.row..);

        let end_content = self.lines[end.row].drain(..=end.row).collect::<String>();
        self.lines[start.row].push_str(&end_content);

        for _ in start.row + 1..=end.row {
            self.lines.remove(start.row + 1);
        }
    }

    pub fn insert_line(&mut self, row: usize) {
        self.lines.insert(row, String::new())
    }

    pub fn col_bound(&self, row: usize, mode: Mode) -> usize {
        match mode {
            Mode::Insert => self.lines[row].len(),
            _ => max(self.lines[row].len() as isize - 1, 0) as usize,
        }
    }

    pub fn row_bound(&self) -> usize {
        max(self.lines.len() as isize - 2, 0) as usize
    }
}
