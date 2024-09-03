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
            })
            .map_err(|e| e.to_string())?;
        // Present target to canvas, keep drawing directly on canvas.
        app.present_render_target()?;
        font.color = Color::WHITE;
        font.draw(
            "This text is being drawn directly to canvas, which means it won't scale automatically...",
            20,
            20,
            scl,
            &mut app.canvas,
        )?;
        // Present canvas
        app.finish_frame()?;
    }
    Ok(())
}
