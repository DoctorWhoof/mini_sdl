### A simple wrapper around SDL2.

This is an early publish of a work-in-progress crate. Use with discretion.

Designed primarily to provide raw access to an RGB pixel buffer and display its contents correctly, with timing options like Vsync and frame rate limiting, and scaling options like Aspect Ratio and integer scaling.

I decided to make this crate after spending an entire weekend testing multiple other libraries that could potentially perform this task, but failed to either perform well or present the frames with proper frame pacing. SDL2 was, the only library that satisfied all requirements, but turned out to be the most complext to use!

The main goal is to favor simplicity over features. With that said, it will expand in the future, providing access to other basic SDL2 features like sound.

*Warning*: Some SDL2 internals are exposed as public members, but this kind of access is *untested*.

## Example

```rust
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
```
