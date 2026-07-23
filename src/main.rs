use std::{panic, process::Command};
use ti::{app::*, render_buffer::RenderBuffer, *};

fn main() -> anyhow::Result<()> {
    let mut app = App::new()?;

    panic::set_hook(Box::new(|panic_info| {
        print!("{CLEAR_SCREEN}\x1b[2 q");
        Command::new("stty").arg("sane").status().unwrap();

        println!("🚨 Fuck! Some shit happened.");

        if let Some(location) = panic_info.location() {
            println!(
                "On this file: '{}', line: {}",
                location.file(),
                location.line()
            );
        }

        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            println!("Panic message: {s}");
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            println!("Panic message: {s}");
        }
    }));

    Command::new("stty").args(["raw", "-echo"]).status()?;
    print!("{CLEAR_SCREEN}");

    let mut front_buffer = RenderBuffer::new();
    let mut back_buffer: RenderBuffer;

    loop {
        back_buffer = RenderBuffer::from(&app.doc, &app.viewport);

        let diff = back_buffer.diff(&front_buffer);

        prin!("{}", RenderBuffer::patch(diff));

        front_buffer = back_buffer.to_owned();

        if !app.handle_input()? {
            break;
        }
    }

    print!("{CLEAR_SCREEN}");
    Command::new("stty").arg("sane").status()?;

    Ok(())
}
