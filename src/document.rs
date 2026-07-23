// TODO: create document struct
// Its responsability is handle the reading, updating, and writing file document

use std::{
    env, fs,
    io::{self},
};

use crate::error::TiError;

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
        fs::write(&self.file_path, self.lines.join("\n"))?;

        Ok(())
    }

    pub fn insert_char(&mut self, col: usize, row: usize, ch: char) {
        self.lines[row].insert(col, ch);
    }
}
