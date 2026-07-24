// TODO: create the viewport #[derive(Debug)]
// Its responsability is to define the boundaries of document rendering on terminal

use crate::terminal::get_terminal_size;

pub struct Viewport {
    pub top_row: usize,
    pub left_column: usize,
    pub width: usize,
    pub height: usize,
}

impl Viewport {
    pub fn new() -> Self {
        let (width, height) = get_terminal_size();

        Self {
            top_row: 0,
            left_column: 0,
            width,
            height,
        }
    }
}
