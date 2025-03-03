use mini_sdl::*;
use sdl2::pixels::Color;

fn main() -> SdlResult {
    let mut app = mini_sdl::App::new(
        "test",
        320,
        240,
        Timing::VsyncLimitFPS(60.0),
        Scaling::PreserveAspect,
        48000,
    )?;

    println!("Current dir is:{:?}", std::env::current_dir());
    println!("Please run this example from the mini_sdl root using 'cargo run -p example_font'");
    println!("Otherwise the font file will not be found!");
    let mut font = app.font_load("example_font/src/classic-display/classic-display.ttf", 16)?;

    let mut buttons = Vec::<Button>::new();

    while !app.quit_requested {
        app.frame_start()?;

        // Test "just_pressed" and "just_released"
        if app.gamepad.is_just_pressed(Button::A) {
            println!("A just pressed");
        }
        if app.gamepad.is_just_released(Button::A) {
            println!("A just released");
        }

        // Clear last frame's button results.
        buttons.clear();
        // Let's start a state with only the right-most bit set to 1
        let mut state: u16 = 0b_0000_0000_0000_0001;
        // The we iterate all bits to the left, each one stores a different button's state
        for _ in 0..16 {
            // Compare to actual gamepad state
            if state & app.gamepad.state() != 0 {
                buttons.push(Button::from(state))
            }
            // But shift to the left for the next iteration
            state <<= 1;
        }

        // Draw to render_target
        app.canvas
            .with_texture_canvas(&mut app.render_target, |target| {
                target.set_draw_color((10, 35, 50, 255));
                target.clear();
                target.set_draw_color((0, 0, 0, 255));
                font.color = Color::RGB(245, 250, 255);
                // Text
                let mut y = 22;
                font.draw("Current buttons:", 20, y, 2.0, target).ok();
                for button in &buttons {
                    y += 22;
                    font.draw(format!("{:?}", button), 20, y, 2.0, target).ok();
                }
            })
            .map_err(|e| e.to_string())?;
        // Present target to canvas, keep drawing directly on canvas.
        app.render_target_present()?;
        // Present canvas
        app.frame_finish()?;
    }
    Ok(())
}
