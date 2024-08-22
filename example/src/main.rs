//! A minimal app using the mini_sdl crate. Simply draws an orange frame with
//! fixed aspect ratio and vsync. The frame rate is printed every second.

use mini_sdl::*;

fn main() -> Result<(), String> {
    let mut app = mini_sdl::App::new(
        "test",
        320,
        240,
        Timing::VsyncLimitFPS(60.0),
        Scaling::PreserveAspect,
    )?;

    app.print_fps = true;

    while !app.quit_requested {
        app.start_frame()?;
        // When calling update_pixels, "buffer" receives access to the
        // render_target pixels in RGB format.
        // "_pitch"", not used here, is the length in bytes of a row of pixels
        app.update_pixels(
            |buffer: &mut [u8], _pitch: usize| {
                let mut i = 0;
                while i < buffer.len() {
                    buffer[i] = 255; // Red
                    buffer[i + 1] = 128; // Green
                    buffer[i + 2] = 16; // Blue
                    i += 3;
                }
            }
        )?;
        app.present_pixel_buffer()?;
        app.finish_frame()?;
    }
    Ok(())
}
