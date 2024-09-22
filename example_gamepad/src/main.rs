use mini_sdl::*;
use sdl2::pixels::Color;

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
    let mut font = app.font_load("example_font/src/classic-display/classic-display.ttf", 16)?;

    while !app.quit_requested {
        app.frame_start()?;
        let state = app.gamepad.state_to_str();
        if app.gamepad.is_just_pressed(Button::A) {
            println!("A just pressed");
        }
        if app.gamepad.is_just_released(Button::A) {
            println!("A just released");
        }
        // Draw to render_target
        app.canvas
            .with_texture_canvas(&mut app.render_target, |target| {
                target.set_draw_color((10, 35, 50, 255));
                target.clear();
                target.set_draw_color((0, 0, 0, 255));
                font.color = Color::RGB(245, 250, 255);
                font.draw(state, 20, 20, 2.0, target).ok();
            })
            .map_err(|e| e.to_string())?;
        // Present target to canvas, keep drawing directly on canvas.
        app.render_target_present()?;
        // Present canvas
        app.frame_finish()?;
    }
    Ok(())
}
