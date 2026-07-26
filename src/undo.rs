use crate::{app::Mode, cursor::Cursor};

const MAX_SIZE: usize = 30;

pub struct UndoStack {
    snapshots: Vec<(Vec<String>, Cursor, Mode)>,
}

impl UndoStack {
    pub fn new() -> Self {
        Self { snapshots: vec![] }
    }

    pub fn push(&mut self, lines: Vec<String>, cursor: Cursor, mode: Mode) {
        if self.snapshots.len() >= MAX_SIZE {
            self.snapshots.remove(0);
        }

        self.snapshots.push((lines, cursor, mode));
    }

    pub fn pop(&mut self) -> Option<(Vec<String>, Cursor, Mode)> {
        self.snapshots.pop()
    }
}
