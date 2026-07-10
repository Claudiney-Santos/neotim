use std::{fs, io};

use crate::screen::ScreenBuffer;

pub fn save(file_path: &str, screen: &ScreenBuffer) -> io::Result<()> {
    let mut content = String::new();
    for i in 0..screen.line_count {
        for j in 0..screen.line_len(i) {
            let char = match screen.cells[i * screen.width + j].char {
                '·' => ' ',
                c => c,
            };

            content.push(char);
        }
        content.push('\n');
    }

    // fs::write(&format!("{}.copy", file_path), content.clone())?;
    fs::write(file_path, content)
}
