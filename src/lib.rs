#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/readme.md"))]

mod font_atlas;
mod gamepad;
mod scaling;
mod timing;

pub use font_atlas::FontAtlas;
pub use gamepad::{Button, GamePad};
pub use scaling::Scaling;
pub use timing::Timing;

pub use sdl2;

const LOWER_ELAPSED_LIMIT:f64 = 1.0 / 360.0;     // 3X 120Hz, 6X 60Hz

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    rect::{Point, Rect},
    render::{Canvas, Texture, TextureAccess, TextureCreator},
    ttf::Sdl2TtfContext,
    video::{Window, WindowContext},
    Sdl,
};
use std::{
    path::Path,
    time::{Duration, Instant},
};

/// A struct that provides SDL initialization and stores the SDL context and its associated data.
/// Designed mostly to be used as a fixed resolution "virtual pixel buffer", but the SDL canvas is
/// available as one of its fields and can be directly manipulated.
pub struct App {
    /// Set to true to quit App on the next update.
    pub quit_requested: bool,
    /// Tiny struct that contains the state of a virtual Gamepad.
    pub gamepad: GamePad,
    /// Minimum sleep time when limiting fps. The smaller it is, the more accurate it will be,
    /// but some platforms (Windows...) seem to struggle with that.
    pub idle_increments_microsecs: u64,
    /// Performs quantization on the elapsed time, rounding it to the nearest most like display
    /// frequency (i.e 60Hz, 72Hz, 90Hz, 120Hz, etc.). This allows for smooth, predictable delta-timing,
    /// especially in pixel art games where precise 1Px increments per frame are common.
    pub smooth_elapsed_time: bool,
    /// Prints every f32 seconds to the terminal the current FPS value.
    pub print_fps_interval: Option<f32>,
    /// Background color
    pub bg_color: (u8, u8, u8, u8),
    ///
    // SDL
    /// The internal SDL canvas. It is automatically cleared on every frame start.
    pub canvas: Canvas<Window>,
    /// The internal SDL texture creator associated with the canvas.
    pub texture_creator: TextureCreator<WindowContext>,
    /// The internal SDL context.
    pub context: Sdl,
    /// The SDL TTF context
    pub fonts: Sdl2TtfContext,
    /// The render target with the fixed resolution specified when creating the app.
    /// This is slower than the pixel buffer if your goal is to draw pixel-by-pixel
    /// (use 'update_pixels' for that) but can use regular SDL drawing functions via
    /// "canvas.with_texture_canvas".
    pub render_target: Texture,
    render_texture: Texture,
    // Video
    width: u32,
    height: u32,
    dpi_mult: f32,
    timing: Timing,
    scaling: Scaling,
    // Timing,
    app_time: Instant,
    last_second: Instant,
    frame_start: Instant,
    update_time: f64,  // Elapsed time before presenting the canvas
    elapsed_time: f64, // Whole frame time at current FPS
    // Overlay
    /// Provides a default FontAtlas for the overlay.
    pub default_font: Option<FontAtlas>,
    /// Scales the distance from line to line.
    pub overlay_line_spacing: f32,
    /// Scales the rendering of the entire overlay text.
    pub overlay_scale: f32,
    /// Initial coordinates (left, top) of the overlay text.
    pub overlay_coords: Point,
    overlay: Vec<String>,
}

impl App {
    /// Returns a new App with a fixed size pixel buffer.
    pub fn new(
        name: &str,
        width: u32,
        height: u32,
        timing: Timing,
        scaling: Scaling,
    ) -> Result<Self, String> {
        let context = sdl2::init()?;
        let video_subsystem = context.video()?;
        let window = video_subsystem
            .window(name, width * 2, height * 2)
            .allow_highdpi()
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = match timing {
            Timing::Vsync | Timing::VsyncLimitFPS(_) => {
                window.into_canvas().accelerated().present_vsync()
            }
            Timing::Immediate | Timing::ImmediateLimitFPS(_) => window.into_canvas().accelerated(),
        }
        .build()
        .map_err(|e| e.to_string())?;

        use sdl2::sys::SDL_WindowFlags::*;
        let dpi_mult = if (canvas.window().window_flags() & SDL_WINDOW_ALLOW_HIGHDPI as u32) != 0 {
            2.0
        } else {
            1.0
        };

        let texture_creator = canvas.texture_creator();
        let render_texture = texture_creator
            .create_texture(
                Some(PixelFormatEnum::RGB24),
                TextureAccess::Streaming,
                width,
                height,
            )
            .ok()
            .unwrap();

        let render_target = texture_creator
            .create_texture(
                Some(PixelFormatEnum::RGB24),
                TextureAccess::Target,
                width,
                height,
            )
            .ok()
            .unwrap();

        let fonts = sdl2::ttf::init().map_err(|e| e.to_string())?;

        Ok(Self {
            quit_requested: false,
            gamepad: GamePad::new(),
            idle_increments_microsecs: 100,
            smooth_elapsed_time: true,
            print_fps_interval: None,
            bg_color: (0, 0, 0, 255),
            app_time: Instant::now(),
            last_second: Instant::now(),
            frame_start: Instant::now(),
            update_time: 0.0,
            elapsed_time: 0.0,
            width,
            height,
            dpi_mult,
            timing,
            scaling,
            canvas,
            render_texture,
            render_target,
            context,
            texture_creator,
            fonts,
            default_font: None,
            overlay: Vec::with_capacity(100),
            overlay_line_spacing: 1.0,
            overlay_scale: 1.0,
            overlay_coords: Point::new(16, 16),
        })
    }

    /// The amount of time in seconds each frame takes to update and draw.
    /// Necessary to correctly implement delta timing, if you wish to do so.
    pub fn elapsed_time(&self) -> f64 {
        self.elapsed_time
    }

    /// The current frame rate.
    pub fn fps(&self) -> f64 {
        1.0 / self.elapsed_time
    }

    /// Time in seconds since the start of the app
    pub fn time(&self) -> f32 {
        self.app_time.elapsed().as_secs_f32()
    }

    /// Adds a line of text to the overlay. The overlay text is cleared on every frame.
    pub fn overlay_push(&mut self, text: impl Into<String>) {
        self.overlay.push(text.into());
    }

    /// Loads a new font
    pub fn load_font<P>(&mut self, path: P, size: u16) -> Result<FontAtlas, String>
    where
        P: AsRef<Path>,
    {
        FontAtlas::new(path, size, &self.fonts, &mut self.texture_creator)
    }

    /// Required at the start of a frame loop, performs basic timing math, clears the canvas and
    /// updates self.gamepad with the current values.
    pub fn start_frame(&mut self) -> Result<(), String> {
        // Whole frame time. Quantized to a minimum interval
        self.elapsed_time = self.frame_start.elapsed().as_secs_f64();
        if self.smooth_elapsed_time {
            self.elapsed_time = quantize(
                self.elapsed_time,
                LOWER_ELAPSED_LIMIT,
            );
            match self.timing {
                Timing::VsyncLimitFPS(limit) | Timing::ImmediateLimitFPS(limit) => {
                    let fps_limit = 1.0 / limit;
                    self.elapsed_time = self.elapsed_time.clamp(fps_limit, 1.0);
                },
                _ => {}
            }
        }

        self.frame_start = Instant::now();

        // Detects new second, prints FPS
        if let Some(interval) = self.print_fps_interval {
            if self.last_second.elapsed().as_secs_f32() > interval {
                self.last_second = Instant::now();
                println!("FPS: {:.1}", (1.0 / self.elapsed_time));
            }
        }

        self.gamepad.set_previous_state();
        for event in self.context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. } => self.quit_requested = true,
                Event::KeyDown {
                    keycode, repeat, ..
                } => {
                    if repeat {
                        continue;
                    }
                    match keycode {
                        None => {}
                        Some(Keycode::Up) => self.gamepad.set(Button::Up, true),
                        Some(Keycode::Down) => self.gamepad.set(Button::Down, true),
                        Some(Keycode::Left) => self.gamepad.set(Button::Left, true),
                        Some(Keycode::Right) => self.gamepad.set(Button::Right, true),
                        Some(Keycode::X) => self.gamepad.set(Button::A, true),
                        Some(Keycode::Z) => self.gamepad.set(Button::B, true),
                        Some(Keycode::S) => self.gamepad.set(Button::X, true),
                        Some(Keycode::A) => self.gamepad.set(Button::Y, true),
                        Some(Keycode::NUM_1) => self.gamepad.set(Button::LeftTrigger, true),
                        Some(Keycode::Q) => self.gamepad.set(Button::LeftShoulder, true),
                        Some(Keycode::NUM_2) => self.gamepad.set(Button::RightTrigger, true),
                        Some(Keycode::W) => self.gamepad.set(Button::RightShoulder, true),
                        Some(Keycode::TAB) => self.gamepad.set(Button::Select, true),
                        Some(Keycode::RETURN) => self.gamepad.set(Button::Select, true),
                        Some(_) => {} // ignore the rest
                    }
                }
                Event::KeyUp {
                    keycode, repeat, ..
                } => {
                    if repeat {
                        continue;
                    }
                    match keycode {
                        None => {}
                        Some(Keycode::Up) => self.gamepad.set(Button::Up, false),
                        Some(Keycode::Down) => self.gamepad.set(Button::Down, false),
                        Some(Keycode::Left) => self.gamepad.set(Button::Left, false),
                        Some(Keycode::Right) => self.gamepad.set(Button::Right, false),
                        Some(Keycode::X) => self.gamepad.set(Button::A, false),
                        Some(Keycode::Z) => self.gamepad.set(Button::B, false),
                        Some(Keycode::S) => self.gamepad.set(Button::X, false),
                        Some(Keycode::A) => self.gamepad.set(Button::Y, false),
                        Some(Keycode::NUM_1) => self.gamepad.set(Button::LeftTrigger, false),
                        Some(Keycode::Q) => self.gamepad.set(Button::LeftShoulder, false),
                        Some(Keycode::NUM_2) => self.gamepad.set(Button::RightTrigger, false),
                        Some(Keycode::W) => self.gamepad.set(Button::RightShoulder, false),
                        Some(Keycode::TAB) => self.gamepad.set(Button::Select, false),
                        Some(Keycode::RETURN) => self.gamepad.set(Button::Select, false),
                        Some(_) => {} // ignore the rest
                    }
                }
                _ => {}
            }
        }
        self.canvas.set_draw_color(self.bg_color);
        self.canvas.clear();
        self.canvas.set_draw_color((255, 255, 255, 255));
        Ok(())
    }

    /// Uses SDL's "texture.with_lock" function to access the pixel buffer as an RGB array.
    pub fn update_pixels<F, R>(&mut self, func: F) -> Result<(), String>
    where
        F: FnOnce(&mut [u8], usize) -> R,
    {
        if let Err(text) = self.render_texture.with_lock(None, func) {
            return Err(text);
        }
        Ok(())
    }

    fn get_scaled_rect(&self) -> Option<Rect> {
        match self.scaling {
            Scaling::Integer | Scaling::PreserveAspect => {
                let window_size = self.canvas.window().size();
                let scale = match self.scaling {
                    Scaling::Integer => (window_size.1 as f32 / self.height as f32).floor(),
                    Scaling::PreserveAspect => window_size.1 as f32 / self.height as f32,
                    _ => 1.0,
                };
                let new_width = self.width as f32 * scale;
                let new_height = self.height as f32 * scale;
                let gap_x = ((window_size.0 as f32 - new_width) * self.dpi_mult) / 2.0;
                let gap_y = ((window_size.1 as f32 - new_height) * self.dpi_mult) / 2.0;
                Some(Rect::new(
                    gap_x as i32,
                    gap_y as i32,
                    (new_width * self.dpi_mult) as u32,
                    (new_height * self.dpi_mult) as u32,
                ))
            }
            Scaling::StretchToWindow => None,
        }
    }

    /// Presents the current pixel buffer respecting the scaling strategy.
    pub fn present_pixel_buffer(&mut self) -> Result<(), String> {
        let rect = self.get_scaled_rect();
        self.canvas
            .copy_ex(&self.render_texture, None, rect, 0.0, None, false, false)?;
        Ok(())
    }

    /// Presents the render target to the canvas respecting the scaling strategy.
    /// Warning: can be much slower than "present_pixel_buffer" if the goal is to simply
    /// draw pixel-by-pixel.
    pub fn present_render_target(&mut self) -> Result<(), String> {
        let rect = self.get_scaled_rect();
        self.canvas
            .copy_ex(&self.render_target, None, rect, 0.0, None, false, false)?;
        Ok(())
    }

    /// Required to be called at the end of a frame loop. Presents the canvas and performs an idle wait
    /// if frame rate limiting is required. Ironically, performing this idle loop may *lower* the CPU
    /// use in some platforms, compared to pure VSync!
    pub fn finish_frame(&mut self) -> Result<(), String> {
        self.update_time = self
            .frame_start
            .elapsed()
            .as_secs_f64()
            .clamp(0.0, 1.0 / 30.0);

        // Overlay
        if let Some(font) = &mut self.default_font {
            // self.canvas.set_draw_color((255, 255, 255, 255));
            let mut y = self.overlay_coords.y;
            for line in &self.overlay {
                font.draw(
                    line,
                    self.overlay_coords.x,
                    y,
                    self.overlay_scale,
                    &mut self.canvas,
                )?;
                let inc = (font.height() as f32 * self.overlay_line_spacing) * self.overlay_scale;
                y += inc as i32;
            }
        }
        self.overlay.clear();

        match self.timing {
            // Optional FPS limiting
            Timing::VsyncLimitFPS(fps) | Timing::ImmediateLimitFPS(fps) => {
                let update_so_far = self.frame_start.elapsed().as_secs_f64();
                let target_time = 1.0 / fps;
                if update_so_far < target_time {
                    // Ideal elapsed time to maintain frame rate
                    let mut idle_time = target_time - update_so_far;
                    // Adjust to increase odds idle loop ends before vsync
                    if let Timing::VsyncLimitFPS(_) = self.timing {
                        idle_time += 0.0001
                    }
                    // Idle loop, wait in small sleep increments until target idle time is reached
                    let mut elapsed = self.frame_start.elapsed().as_secs_f64();
                    while elapsed < idle_time {
                        std::thread::sleep(Duration::from_micros(self.idle_increments_microsecs));
                        elapsed = self.frame_start.elapsed().as_secs_f64();
                    }
                }
            }
            // Vsync or Immediate don't sleep
            _ => {}
        };

        self.canvas.present();
        Ok(())
    }
}

#[inline(always)]
fn quantize(value: f64, size: f64) -> f64 {
    (value / size).round() * size
}

// CAn't figure out how to test with SDL! This doesn't work
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn create_window() {
//         let mut app = App::new(
//             "test",
//             320,
//             240,
//             Timing::VsyncLimitFPS(60.0),
//             Scaling::PreserveAspect,
//         ).unwrap();
//         app.start_frame().unwrap();
//         app.present_pixel_buffer().unwrap();
//         app.finish_frame().unwrap();
//     }
// }
