use mini_sdl::{
    sdl3::{pixels::Color, render::FRect},
    *,
};

fn main() -> SdlResult<()> {
    let mut app = mini_sdl::App::new(
        "test",
        320,
        240,
        Timing::VsyncLimitFPS(60.0),
        Scaling::PreserveAspect
    )?;
    app.init_render_target()?;

    println!("Current dir is:{:?}", std::env::current_dir());
    println!("Please run this example from the mini_sdl root using 'cargo run -p example_font'");
    println!("Otherwise the font file will not be found!");

    let mut f1 = app.font_load("example_overlay/src/roboto_medium.ttf", 24.0, 1.0)?;
    let mut f2 = app.font_load("example_overlay/src/roboto_medium.ttf", 12.0, 1.0)?;

    while !app.quit_requested {
        app.frame_start()?;
        // Draw to render_target
        let Some(render_target) = &mut app.render_target else {
            println!("Render target not found");
            break
        };
        app.canvas
            .with_texture_canvas(render_target, |target| {
                target.set_draw_color((38, 36, 35, 255));
                target.clear();
                target.set_draw_color((0, 0, 0, 255));
                // Unfortunately SDL's error type here is different, and we can't use the '?' operator.
                target.draw_rect(FRect::new(10.0, 20.0, 300.0, 200.0)).ok();
                f1.color = Color::RGB(255, 230, 150);
                f2.draw("But this text is drawn", 20, 50, 1.0, target).ok();
                f2.draw("to the render target", 20, 65, 1.0, target).ok();
                f2.draw("and scales accordingly!", 20, 80, 1.0, target).ok();
                f2.draw("0123456789", 20, 120, 1.0, target).ok();
                f2.draw("ABCDEFGHIJKLM", 20, 135, 1.0, target).ok();
                f2.draw("NOPQRSTUVWXYZ", 20, 150, 1.0, target).ok();
                f2.draw("!@#$%^&*()-_=+,.:;'~? ", 20, 165, 1.0, target).ok();
            })?;
        // Present target to canvas, keep drawing directly on canvas.
        app.render_target_present()?;
        f1.color = Color::WHITE;
        f1.draw(
            "This text is being drawn directly to canvas, which means it won't scale automatically...",
            20,
            20,
            1.0,
            &mut app.canvas,
        )?;

        // Optional, display font texture
        // let query = f1.texture.query();
        // let rect = sdl3::rect::Rect::new(0, 0, query.width * 2, query.height * 2);
        // app.canvas.copy(&f1.texture, None, rect)?;

        // Present canvas
        app.frame_finish()?;
    }
    Ok(())
}
