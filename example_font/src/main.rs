use mini_sdl::*;
use sdl2::{pixels::Color, rect::Rect};

fn main() -> Result<(), String> {
    let mut app = mini_sdl::App::new(
        "test",
        320,
        240,
        Timing::VsyncLimitFPS(60.0),
        Scaling::PreserveAspect,
    )?;

    println!("Current dir is:{:?}", std::env::current_dir());
    println!("Please run this example from the mini_sdl root using 'cargo run -p example_font'");
    println!("Otherwise the font file will not be found!");

    let mut font = app.load_font("example_font/classic-display/classic-display.ttf", 16)?;

    while !app.quit_requested {
        app.start_frame()?;
        let scl = 5.0;

        font.color = Color::WHITE;
        font.draw("This is a test: ABCDEFG", 20, 20, scl, &mut app.canvas)?;
        font.color = Color::YELLOW;
        font.draw("0123456789", 20, 100, scl, &mut app.canvas)?;
        font.color = Color::RGB(255, 200, 0);
        font.draw("The Brown Fox jumps over the fence", 20, 180, scl, &mut app.canvas)?;
        font.color = Color::WHITE;
        font.draw("This is the entire FontAtlas texture:", 20, 400, scl, &mut app.canvas)?;

        let query = font.texture.query();
        let dest = Rect::new(20, 460, query.width * scl as u32, query.height * scl as u32);
        app.canvas.copy(&font.texture, None, dest)?;

        app.finish_frame()?;
    }
    Ok(())
}
