# A simple wrapper around SDL3.

This is an early publish of a work-in-progress crate. Use with discretion.

### This crate assumes you've already installed SDL3 on your system (i.e. 'brew install sdl3' on MacOS). It wil not run if SDL3 isn't found.

Designed primarily to provide raw access to an RGB pixel buffer and display its contents correctly, with timing options like Vsync and frame rate limiting, and scaling options like Aspect Ratio and integer scaling.

I decided to make this crate after spending an entire weekend testing multiple libraries that could potentially perform this task, but failed to either perform well or present the frames with proper frame pacing. SDL3 was the only high-level library that satisfied all requirements, but turned out to be the most complex to use, so wrapping it in something much simpler can be of value to many people who just want a "framebuffer" to write pixels to.

The main goal is to favor simplicity over features. With that said, it will expand in the future, providing access to other basic SDL3 features like sound.

_Warning_: Some SDL3 internals are exposed as public members, but this kind of access is _untested_.

## Example

This example can be run invoking `cargo run -p example` from a terminal at the root of the crate.

```rust
use mini_sdl::*;

fn main() -> SdlResult<()> {
    let time = std::time::Instant::now();
    let mut app = mini_sdl::App::new(
        "test",                         // App Name displayed on window.
        320,                            // Pixel buffer resolution, won't be used if you draw directly to canvas.
        240,
        Timing::VsyncLimitFPS(60.0),    // Frame Timing strategy.
        Scaling::PreserveAspect,        // Pixel buffer scaling strategy, ignored if you draw directly to canvas.
        None                            // Audio sampling rate, if audio is desired.
    )?;

    app.print_fps_interval = Some(1.0);

    while !app.quit_requested {
        app.frame_start()?;
        // When calling pixel_buffer_update, "buffer" receives access
        // to the render_target pixels in RGB format.
        // _pitch, not used here, is how many bytes per row.
        app.pixel_buffer_update(
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
        app.pixel_buffer_present()?;
        app.frame_finish()?;
        // Quit after 2 seconds
        if time.elapsed().as_secs_f64() > 2.0 {
            app.quit_requested = true
        }
    }
    Ok(())
}
```

### Static Builds

To build statically, run the 'cargo build' command preceded by these flags which will point out where "sdl3" and "SDL3_ttf" are:

```bash
RUSTFLAGS='-L /opt/homebrew/Cellar/sdl3/2.30.8/lib -L /opt/homebrew/Cellar/SDL3_ttf/2.22.0/lib' cargo build
```

Replace the paths with ones valid for your system, then add the "static-link" feature to cargo.toml.
