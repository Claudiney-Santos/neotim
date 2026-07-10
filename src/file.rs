use std::{fs, io};

use crate::screen::ScreenBuffer;

pub fn save(file_path: &str, screen: &ScreenBuffer) -> io::Result<()> {
    let mut content = String::new();
    let mut i = 0;
    while i < screen.width * screen.height {
        let (ch, add) = match screen.cells[i].char {
            ' ' => ('\n', screen.width - i % screen.width),
            '·' => (' ', 1),
            c => (c, 1),
        };

        content.push(ch);
        i += add;
    }

    // fs::write(&format!("{}.copy", file_path), content.clone())?;
    fs::write(file_path, content)
}
