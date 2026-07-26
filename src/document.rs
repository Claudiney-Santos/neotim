// TODO: create document struct
// Its responsability is handle the reading, updating, and writing file document

use std::cmp::max;
use std::{
    env, fs,
    io::{self},
};

use crate::{app::Mode, error::TiError};

#[derive(Clone, Copy, PartialEq, Debug)]
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

    pub fn insert(&mut self, pos: Pos, str: &str) {
        let mut it = str.split("\n");

        let rest = self.lines[pos.row].drain(pos.col..).collect::<String>();

        it.next()
            .map(|s| self.lines[pos.row].insert_str(pos.col, s));

        it.enumerate().for_each(|(i, s)| {
            self.lines.insert(pos.row + i + 1, s.to_owned());
        });

        self.lines[pos.row + str.matches("\n").count()].push_str(&rest);
    }

    pub fn delete(&mut self, start: Pos, end: Pos) {
        assert!(
            start.row < end.row || (start.row == end.row && start.col <= end.col),
            "You messed up with start and end boundaries"
        );

        let del_start = start.col == self.lines[start.row].len();
        let del_end = end.col == self.lines[end.row].len();

        if start.row == end.row {
            if !del_start {
                self.lines[start.row].drain(start.col..end.col + if del_end { 0 } else { 1 });
            }

            if del_end && start.row + 1 < self.lines.len() {
                let next_line = self.lines.remove(start.row + 1);
                self.lines[start.row].push_str(&next_line);
            }
            return;
        }

        if !del_start {
            self.lines[start.row].drain(start.col..);
        }

        self.lines[end.row].drain(..end.col + if del_end { 0 } else { 1 });

        for _ in start.row + 1..end.row + if del_end { 1 } else { 0 } {
            self.lines.remove(start.row + 1);
        }

        if start.row + 1 < self.lines.len() {
            let next_line = self.lines.remove(start.row + 1);
            self.lines[start.row].push_str(&next_line);
        }
    }

    pub fn insert_line(&mut self, row: usize) {
        self.lines.insert(row, String::new())
    }

    pub fn col_bound(&self, row: usize, mode: Mode) -> usize {
        match mode {
            Mode::Insert | Mode::Visual(_) => self.lines[row].len(),
            _ => max(self.lines[row].len() as isize - 1, 0) as usize,
        }
    }

    pub fn row_bound(&self) -> usize {
        max(self.lines.len() as isize - 2, 0) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_joinlines_correctly() {
        let mut doc = Document {
            file_path: "monster.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "asdf".to_owned()],
        };

        let pos = Pos { row: 0, col: 4 };

        doc.delete(pos.clone(), pos);

        assert_eq!(doc.lines, vec!["asdfasdf".to_owned()]);
    }

    #[test]
    fn it_delete_content_in_the_same_line_without_join() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "asdf".to_owned()],
        };

        let start = Pos { row: 0, col: 0 };
        let end = Pos { row: 0, col: 2 };

        doc.delete(start, end);

        assert_eq!(doc.lines, vec!["f".to_owned(), "asdf".to_owned()]);
    }

    #[test]
    #[should_panic]
    fn it_should_crash() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "asdf".to_owned()],
        };

        let start = Pos { row: 0, col: 10 };
        let end = Pos { row: 0, col: 2 };

        doc.delete(start, end);
    }

    #[test]
    fn it_delete_content_in_the_same_line_and_join() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "asdf".to_owned()],
        };

        let start = Pos { row: 0, col: 2 };
        let end = Pos { row: 0, col: 4 };

        doc.delete(start, end);

        assert_eq!(doc.lines, vec!["asasdf".to_owned()]);
    }

    #[test]
    fn it_delete_multiple_lines() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
            ],
        };

        let start = Pos { row: 0, col: 4 };
        let end = Pos { row: 2, col: 4 };

        doc.delete(start, end);

        assert_eq!(doc.lines, vec!["asdfasdf".to_owned()]);
    }

    #[test]
    fn it_delete_multiple_lines_and_drain() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
            ],
        };

        let start = Pos { row: 0, col: 4 };
        let end = Pos { row: 2, col: 2 };

        doc.delete(start, end);

        assert_eq!(doc.lines, vec!["asdff".to_owned(), "asdf".to_owned()]);
    }

    #[test]
    fn it_drain_and_delete_multiple_lines() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
            ],
        };

        let start = Pos { row: 0, col: 2 };
        let end = Pos { row: 2, col: 4 };

        doc.delete(start, end);

        assert_eq!(doc.lines, vec!["asasdf".to_owned()]);
    }

    #[test]
    fn it_drain_and_delete_multiple_lines2() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec![
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
                "asdf".to_owned(),
            ],
        };

        let start = Pos { row: 0, col: 2 };
        let end = Pos { row: 2, col: 1 };

        doc.delete(start, end);

        assert_eq!(doc.lines, vec!["asdf".to_owned(), "asdf".to_owned()]);
    }

    #[test]
    fn it_insert_inline_content() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "----".to_owned()],
        };

        doc.insert(Pos { row: 0, col: 2 }, "mise");

        assert_eq!(doc.lines, vec!["asmisedf".to_owned(), "----".to_owned()]);
    }

    #[test]
    fn it_insert_multiple_line_content() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "----".to_owned()],
        };

        doc.insert(Pos { row: 0, col: 2 }, "mise\n123");

        assert_eq!(
            doc.lines,
            vec!["asmise".to_owned(), "123df".to_owned(), "----".to_owned()]
        );
    }

    #[test]
    fn it_insert_new_line_with_content() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "----".to_owned()],
        };

        doc.insert(Pos { row: 0, col: 4 }, "\n123");

        assert_eq!(
            doc.lines,
            vec!["asdf".to_owned(), "123".to_owned(), "----".to_owned()]
        );
    }

    #[test]
    fn it_insert_content_and_new_line() {
        let mut doc = Document {
            file_path: "ti.ti".to_owned(),
            lines: vec!["asdf".to_owned(), "----".to_owned()],
        };

        doc.insert(Pos { row: 0, col: 4 }, "123\n");

        assert_eq!(
            doc.lines,
            vec!["asdf123".to_owned(), "".to_owned(), "----".to_owned()]
        );
    }
}
