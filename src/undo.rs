use crate::{
    cursor::Cursor,
    screen::{Cell, Mode},
};

const UNDO_STACK_SIZE: usize = 10;

pub struct UndoEntry {
    pub delta: Vec<(usize, usize, Cell)>,
    pub cursor: Cursor,
    pub line_count: usize,
}

pub struct UndoStack {
    undos: Vec<UndoEntry>,
    last_mode: Mode,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            undos: vec![],
            last_mode: Mode::Normal,
        }
    }

    pub fn push(&mut self, mode: Mode, mut entry: UndoEntry) {
        if mode == Mode::Undo || entry.delta.last().is_none() {
            return;
        }

        if let (Mode::Insert, Mode::Insert, Some(undo)) =
            (mode, self.last_mode, self.undos.last_mut())
        {
            undo.delta.append(&mut entry.delta);
            return;
        }

        if self.undos.len() >= UNDO_STACK_SIZE {
            self.undos.remove(0);
        }

        self.undos.push(entry);
        self.last_mode = mode;
    }

    pub fn pop(&mut self) -> Option<UndoEntry> {
        self.undos.pop()
    }
}
