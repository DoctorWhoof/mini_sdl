# A simple wrapper around SDL2.

This is an early publish of a work-in-progress crate. Use with discretion.

### This crate assumes you've already installed SDL2 on your system (i.e. 'brew install sdl2' on MacOS). It wil not run if SDL2 isn't found.

Designed primarily to provide raw access to an RGB pixel buffer and display its contents correctly, with timing options like Vsync and frame rate limiting, and scaling options like Aspect Ratio and integer scaling.

I decided to make this crate after spending an entire weekend testing multiple libraries that could potentially perform this task, but failed to either perform well or present the frames with proper frame pacing. SDL2 was the only high-level library that satisfied all requirements, but turned out to be the most complex to use, so wrapping it in something much simpler can be of value to many people who just want a "framebuffer" to write pixels to.

The main goal is to favor simplicity over features. With that said, it will expand in the future, providing access to other basic SDL2 features like sound.

*Warning*: Some SDL2 internals are exposed as public members, but this kind of access is *untested*.

## Example
This example can be run invoking `cargo run -p example` from a terminal at the root of the crate.

```rust
use mini_sdl::*;

fn main() -> SdlResult {
    let mut app = mini_sdl::App::new(
        "test",
        320,
        240,
        Timing::VsyncLimitFPS(60.0),
        Scaling::PreserveAspect,
        44100
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
    }
    Ok(())
}
```

### Static Builds

To build statically, run the 'cargo build' command preceded by these flags which will point out where "sdl2" and "sdl2_ttf" are:

```bash
RUSTFLAGS='-L /opt/homebrew/Cellar/sdl2/2.30.8/lib -L /opt/homebrew/Cellar/sdl2_ttf/2.22.0/lib' cargo build
```

Replace the paths with ones valid for your system, then add the "static-link" feature to cargo.toml.
