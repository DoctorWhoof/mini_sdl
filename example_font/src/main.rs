use mini_sdl::*;
use sdl2::{pixels::Color, rect::Rect};

fn main() -> SdlResult {
    let mut app = mini_sdl::App::new(
        "test",
        320,
        240,
        Timing::VsyncLimitFPS(60.0),
        Scaling::PreserveAspect,
        48000
    )?;

    println!("Current dir is:{:?}", std::env::current_dir());
    println!("Please run this example from the mini_sdl root using 'cargo run -p example_font'");
    println!("Otherwise the font file will not be found!");

    // let mut font = app.font_load("example_font/src/classic-display/classic-display.ttf", 16)?;
    let mut font = app.font_load("example_font/src/classic-mono-narrow/classic-mono-narrow.ttf", 8)?;

    while !app.quit_requested {
        app.frame_start()?;
        let scl = 2.0;
        // Draw to render_target
        app.canvas
            .with_texture_canvas(&mut app.render_target, |target| {
                target.set_draw_color((50, 35, 25, 255));
                target.clear();
                target.set_draw_color((0, 0, 0, 255));
                // Unfortunately SDL's error type here is different, and we can't use the '?' operator.
                target.draw_rect(Rect::new(10, 20, 300, 200)).ok();
                font.color = Color::RGB(255, 230, 150);
                font.draw("But this text is drawn", 20, 50, 2.0, target).ok();
                font.draw("to the render target", 20, 70, 2.0, target).ok();
                font.draw("and scales accordingly!", 20, 90, 2.0, target).ok();
                font.draw("1234567890", 20, 110, 2.0, target).ok();
                font.draw("ABCDEFGHIJKLMNOP", 20, 130, 2.0, target).ok();
                font.draw("QRSTUVWXYZ", 20, 150, 2.0, target).ok();

            })
            .map_err(|e| e.to_string())?;
        // Present target to canvas, keep drawing directly on canvas.
        app.render_target_present()?;
        font.color = Color::WHITE;
        font.draw(
            "This text is being drawn directly to canvas, which means it won't scale automatically...",
            20,
            20,
            scl,
            &mut app.canvas,
        )?;

        // Optional, display font texture
        // let query = font.texture.query();
        // let rect = Rect::new(0, 0, query.width * 4, query.height * 4);
        // app.canvas.copy(&font.texture, None, rect)?;

        // Present canvas
        app.frame_finish()?;
    }
    Ok(())
}
