#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/readme.md"))]

mod scaling;
mod timing;

use sdl3::audio::{AudioFormat, AudioSpec, AudioStreamOwner};
use sdl3::gamepad::Gamepad;
use sdl3::pixels::PixelFormat;
use sdl3::sys::pixels::SDL_PixelFormat;
pub use smooth_buffer::SmoothBuffer;
pub use smooth_buffer::{Float, Num};

pub use padstate::{APad, Button};
pub use scaling::Scaling;

pub use sdl3;
pub use timing::Timing;

#[cfg(feature = "ttf")]
mod font_atlas;
#[cfg(feature = "ttf")]
pub use font_atlas::FontAtlas;

use sdl3::{
    event::Event,
    keyboard::{Keycode, Mod},
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    EventPump, Sdl,
};
use std::time::{Duration, Instant};

pub type SdlResult<E> = Result<E, Box<dyn std::error::Error>>;

const ELAPSED_QUANT_SIZE: f64 = 1.0 / 1440.0; // 3X 120Hz, 6X 60Hz

/// A struct that provides SDL initialization and stores the SDL context and its associated data.
/// Designed mostly to be used as a fixed resolution "virtual pixel buffer", but the SDL canvas is
/// available as one of its fields and can be directly manipulated.
pub struct App {
    /// Set to true to quit App on the next update.
    pub quit_requested: bool,
    /// Tiny struct that contains the state of a virtual Gamepad.
    pub apad: APad,
    /// Minimum sleep time when limiting fps. The smaller it is, the more accurate it will be,
    /// but some platforms (Windows...) seem to struggle with that.
    pub idle_increments_microsecs: u64,
    /// Prints every f32 seconds to the terminal the current FPS value.
    pub print_fps_interval: Option<f32>,
    /// Background color.
    pub bg_color: (u8, u8, u8, u8),
    /// Controls whether text overlay is visible.
    pub display_overlay: bool,
    // SDL
    /// The internal SDL canvas. It is automatically cleared on every frame start.
    pub canvas: Canvas<Window>,
    /// The internal SDL texture creator associated with the canvas.
    pub texture_creator: TextureCreator<WindowContext>,
    /// The internal SDL context.
    pub context: Sdl,
    /// Cache for the event pump
    pub events: EventPump,
    /// Player 1 controller
    pub controller_1: Option<Gamepad>,
    pub allow_analog_to_dpad_x: bool,
    pub allow_analog_to_dpad_y: bool,
    /// The SDL TTF context
    #[cfg(feature = "ttf")]
    pub fonts: sdl3::ttf::Sdl3TtfContext,
    /// The render target with the fixed resolution specified when creating the app.
    /// This is slower than the pixel buffer if your goal is to draw pixel-by-pixel
    /// (use 'pixel_buffer_update' for that) but can use regular SDL drawing functions via
    /// "canvas.with_texture_canvas".
    pub render_target: Texture,
    pixel_buffer: Texture,
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
    update_time_buffer: SmoothBuffer<60, f64>,
    elapsed_time: f64,     // Whole frame time at current FPS.
    elapsed_time_raw: f64, // Elapsed time without quantizing and smoothing.
    // Overlay
    /// Provides a default FontAtlas for the overlay.
    #[cfg(feature = "ttf")]
    pub default_font: Option<FontAtlas>,
    /// Scales the distance from line to line.
    #[cfg(feature = "ttf")]
    pub overlay_line_spacing: f32,
    /// Scales the rendering of the entire overlay text.
    #[cfg(feature = "ttf")]
    pub overlay_scale: f32,
    /// Initial coordinates (left, top) of the overlay text.
    #[cfg(feature = "ttf")]
    pub overlay_coords: sdl3::rect::Point,
    #[cfg(feature = "ttf")]
    overlay: Vec<String>,
    // Audio
    // pub audio_device: Option<AudioStreamWithCallback<AudioPlayback>>,
    // audio_device: Option<AudioDevice>,
    pub audio_stream: Option<AudioStreamOwner>,
    sample_rate: Option<u32>,
    // buffer: Vec<i16>,
}

impl App {
    /// Returns a result with an App with default configuration
    pub fn default() -> SdlResult<App> {
        Self::new(
            "App",
            320,
            240,
            Timing::VsyncLimitFPS(60.0),
            Scaling::PreserveAspect,
            Some(44100),
        )
    }

    /// Returns a result containing a new App with a fixed size pixel buffer.
    pub fn new(
        name: &str,
        width: u32,
        height: u32,
        timing: Timing,
        scaling: Scaling,
        sample_rate: Option<u32>,
    ) -> SdlResult<App> {
        // sdl3::hint::set("SDL_JOYSTICK_THREAD", "1");
        let context = sdl3::init()?;

        // Game controller
        let gamepad_subsystem = context.gamepad()?;
        let available = gamepad_subsystem.num_gamepads()?;
        println!("MiniSDL: {} joysticks available", available);

        // Iterate over all available joysticks and look for game controllers.
        let controller_1 = (0..available).find_map(|id| {
            if !gamepad_subsystem.is_game_controller(id) {
                println!("MiniSDL: {} is not a game controller", id);
                return None;
            }
            match gamepad_subsystem.open(id) {
                Ok(c) => {
                    println!("MiniSDL: Opened joystick {} as \"{}\"", id, c.name());
                    Some(c)
                }
                Err(e) => {
                    println!("MiniSDL: Failed to open joystick: {:?}", e);
                    None
                }
            }
        });

        // Video & Window
        let video_subsystem = context.video()?;
        let window = video_subsystem
            .window(name, width * 2, height * 2)
            .high_pixel_density()
            .position_centered()
            .resizable()
            .build()?;

        let canvas = match timing {
            Timing::Vsync | Timing::VsyncLimitFPS(_) => window.into_canvas(),
            Timing::Immediate | Timing::ImmediateLimitFPS(_) => window.into_canvas(),
        };

        let dpi_mult = 2.0; // TODO: Will need to check in SDL3
                            // use sdl3::sys::SDL_WindowFlags::*;
                            // let dpi_mult = if (canvas.window().window_flags() & WINDOW_ALLOW_HIGHDPI as u32) != 0 {
                            //     2.0
                            // } else {
                            //     1.0
                            // };

        let texture_creator = canvas.texture_creator();

        let pixel_buffer = texture_creator.create_texture_streaming(
            unsafe { PixelFormat::from_ll(SDL_PixelFormat::RGB24) },
            width,
            height,
        )?;

        let render_target = texture_creator.create_texture_target(
            unsafe { PixelFormat::from_ll(SDL_PixelFormat::RGB24) },
            width,
            height,
        )?;

        // let mut audio_device = None;
        let mut audio_stream = None;
        if let Some(sample_rate) = sample_rate {
            //Sound init
            let spec = AudioSpec {
                freq: Some(sample_rate as i32),
                channels: Some(2),
                format: Some(AudioFormat::s16_sys()),
            };
            let audio_subsystem = context.audio()?;

            // let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec)?;

            // let device =
            //     audio_subsystem.open_playback_stream(&desired_spec, AudioPlayback::new())?;
            //
            let device = audio_subsystem.open_playback_device(&spec)?;
            let stream = device.open_device_stream(Some(&spec))?;
            audio_stream = Some(stream);
            // audio_device = Some(device);
        }

        let events = context.event_pump()?;

        Ok(Self {
            quit_requested: false,
            apad: APad::new(),
            idle_increments_microsecs: 100,
            print_fps_interval: None,
            bg_color: (0, 0, 0, 255),
            display_overlay: true,
            app_time: Instant::now(),
            last_second: Instant::now(),
            frame_start: Instant::now(),
            update_time_buffer: SmoothBuffer::pre_filled(1.0 / 120.0),
            elapsed_time: 0.0,
            elapsed_time_raw: 0.0,
            width,
            height,
            dpi_mult,
            timing,
            scaling,
            canvas,
            pixel_buffer,
            render_target,
            context,
            events,
            controller_1,
            allow_analog_to_dpad_x: false,
            allow_analog_to_dpad_y: false,
            texture_creator,
            // Audio
            sample_rate,
            // audio_device,
            audio_stream,
            // Optional features
            #[cfg(feature = "ttf")]
            fonts: sdl3::ttf::init()?,
            #[cfg(feature = "ttf")]
            default_font: None,
            #[cfg(feature = "ttf")]
            overlay: Vec::with_capacity(100),
            #[cfg(feature = "ttf")]
            overlay_line_spacing: 1.0,
            #[cfg(feature = "ttf")]
            overlay_scale: 1.0,
            #[cfg(feature = "ttf")]
            overlay_coords: sdl3::rect::Point::new(16, 16),
        })
    }

    /// The render target width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The render target height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The window width, which is independent from the render target.
    pub fn window_width(&self) -> u32 {
        self.canvas.window().size().0
    }

    /// The window height, which is independent from the render target.
    pub fn window_height(&self) -> u32 {
        self.canvas.window().size().1
    }

    /// The amount of time in seconds each frame takes to update and draw.
    /// Necessary to correctly implement delta timing, if you wish to do so.
    /// Performs quantization, rounding it to the nearest most like display frequency
    /// (i.e 60Hz, 72Hz, 90Hz, 120Hz, etc.). This allows for smooth, predictable delta-timing,
    /// especially in pixel art games where precise 1Px increments per frame are common.
    pub fn elapsed_time(&self) -> f64 {
        self.elapsed_time
    }

    /// The elapsed time without any smoothing or quantization. Can lead to severe frame pacing issues
    /// if used in delta-timing mechanisms.
    pub fn elapsed_time_raw(&self) -> f64 {
        self.elapsed_time_raw
    }

    /// How long the frame took to update before presenting the canvas.
    pub fn update_time(&self) -> f64 {
        self.update_time_buffer.average()
    }

    /// The current frame rate.
    pub fn fps(&self) -> f64 {
        1.0 / self.elapsed_time
    }

    /// Time in seconds since the start of the app
    pub fn time(&self) -> Duration {
        self.app_time.elapsed()
    }

    /// Adds a line of text to the overlay. The overlay text is cleared on every frame.
    #[cfg(feature = "ttf")]
    pub fn overlay_push(&mut self, text: impl Into<String>) {
        self.overlay.push(text.into());
    }

    #[cfg(feature = "ttf")]
    /// Loads a TTF font and converts it to a FontAtlas of fixed size.
    pub fn font_load<P>(&mut self, path: P, size: f32, line_spacing: f32) -> SdlResult<FontAtlas>
    where
        P: AsRef<std::path::Path>,
    {
        FontAtlas::new(
            path,
            size,
            line_spacing,
            &self.fonts,
            &mut self.texture_creator,
        )
    }

    /// Required at the start of a frame loop, performs basic timing math, clears the canvas and
    /// updates self.apad with the current values.
    pub fn frame_start(&mut self) -> SdlResult<()> {
        // Whole frame time.
        self.elapsed_time_raw = self.frame_start.elapsed().as_secs_f64();
        self.elapsed_time = match self.timing {
            Timing::Vsync | Timing::VsyncLimitFPS(_) => {
                // Quantized to a minimum interval to ensure it exactly matches the display
                quantize(self.elapsed_time_raw, ELAPSED_QUANT_SIZE)
            }
            Timing::Immediate | Timing::ImmediateLimitFPS(_) => self.elapsed_time_raw,
        };
        self.frame_start = Instant::now();

        // Input
        self.apad.copy_current_to_previous_state();

        for event in self.events.poll_iter() {
            use padstate::Button as butt;
            use sdl3::gamepad::Button::*;
            match event {
                Event::ControllerAxisMotion {
                    axis,
                    timestamp: _,
                    which: _,
                    value,
                } => {
                    use sdl3::gamepad::Axis::*;
                    const AXIS_DEAD_ZONE: i16 = 8000;
                    match axis {
                        LeftX => {
                            if self.allow_analog_to_dpad_x {
                                self.apad.set_button(butt::Left, value < -AXIS_DEAD_ZONE);
                                self.apad.set_button(butt::Right, value > AXIS_DEAD_ZONE);
                            } else {
                                self.apad.left_stick_x = value;
                            }
                        }
                        LeftY => {
                            if self.allow_analog_to_dpad_y {
                                self.apad.set_button(butt::Up, value < -AXIS_DEAD_ZONE);
                                self.apad.set_button(butt::Down, value > AXIS_DEAD_ZONE);
                            } else {
                                self.apad.left_stick_y = value;
                            }
                        }
                        RightX => {
                            // Could map to additional directional controls if needed
                        }
                        RightY => {
                            // Could map to additional directional controls if needed
                        }
                        TriggerLeft => {
                            self.apad
                                .set_button(butt::LeftTrigger, value > AXIS_DEAD_ZONE);
                        }
                        TriggerRight => {
                            self.apad
                                .set_button(butt::RightTrigger, value > AXIS_DEAD_ZONE);
                        }
                    }
                }
                Event::ControllerButtonDown { button, .. } => match button {
                    DPadUp => self.apad.set_button(butt::Up, true),
                    DPadDown => self.apad.set_button(butt::Down, true),
                    DPadLeft => self.apad.set_button(butt::Left, true),
                    DPadRight => self.apad.set_button(butt::Right, true),
                    South => self.apad.set_button(butt::A, true),
                    East => self.apad.set_button(butt::B, true),
                    West => self.apad.set_button(butt::X, true),
                    North => self.apad.set_button(butt::Y, true),
                    // LeftStick => self.apad.set_button(butt::LeftTrigger, true),
                    LeftShoulder => self.apad.set_button(butt::LeftShoulder, true),
                    // RightStick => self.apad.set_button(butt::RightTrigger, true),
                    RightShoulder => self.apad.set_button(butt::RightShoulder, true),
                    Guide => self.apad.set_button(butt::Menu, true),
                    Start => self.apad.set_button(butt::Start, true),
                    Back => self.apad.set_button(butt::Select, true),
                    _ => {}
                },
                Event::ControllerButtonUp { button, .. } => match button {
                    DPadUp => self.apad.set_button(butt::Up, false),
                    DPadDown => self.apad.set_button(butt::Down, false),
                    DPadLeft => self.apad.set_button(butt::Left, false),
                    DPadRight => self.apad.set_button(butt::Right, false),
                    South => self.apad.set_button(butt::A, false),
                    East => self.apad.set_button(butt::B, false),
                    West => self.apad.set_button(butt::X, false),
                    North => self.apad.set_button(butt::Y, false),
                    LeftStick => self.apad.set_button(butt::LeftTrigger, false),
                    LeftShoulder => self.apad.set_button(butt::LeftShoulder, false),
                    RightStick => self.apad.set_button(butt::RightTrigger, false),
                    RightShoulder => self.apad.set_button(butt::RightShoulder, false),
                    Guide => self.apad.set_button(butt::Menu, false),
                    Start => self.apad.set_button(butt::Start, false),
                    Back => self.apad.set_button(butt::Select, false),
                    _ => {}
                },
                Event::KeyDown {
                    keycode,
                    repeat: false,
                    keymod,
                    ..
                } => {
                    match keycode {
                        Option::None => {}
                        Some(Keycode::Up) => self.apad.set_button(butt::Up, true),
                        Some(Keycode::Down) => self.apad.set_button(butt::Down, true),
                        Some(Keycode::Left) => self.apad.set_button(butt::Left, true),
                        Some(Keycode::Right) => self.apad.set_button(butt::Right, true),
                        Some(Keycode::X) => self.apad.set_button(butt::A, true),
                        Some(Keycode::Z) => self.apad.set_button(butt::B, true),
                        Some(Keycode::A) => self.apad.set_button(butt::Y, true),
                        Some(Keycode::S) => self.apad.set_button(butt::X, true),
                        Some(Keycode::_1) => self.apad.set_button(butt::LeftTrigger, true),
                        Some(Keycode::Q) => self.apad.set_button(butt::LeftShoulder, true),
                        Some(Keycode::_2) => self.apad.set_button(butt::RightTrigger, true),
                        Some(Keycode::W) => self.apad.set_button(butt::RightShoulder, true),
                        Some(Keycode::Tab) => self.apad.set_button(butt::Select, true),
                        Some(Keycode::Return) => self.apad.set_button(butt::Start, true),
                        Some(Keycode::Escape) => self.apad.set_button(butt::Menu, true),
                        Some(Keycode::O) => {
                            if keymod == Mod::LCTRLMOD {
                                self.display_overlay = !self.display_overlay
                            }
                        }
                        Some(_) => {} // ignore the rest
                    }
                }
                Event::KeyUp {
                    keycode,
                    repeat: false,
                    ..
                } => {
                    match keycode {
                        Option::None => {}
                        Some(Keycode::Up) => self.apad.set_button(butt::Up, false),
                        Some(Keycode::Down) => self.apad.set_button(butt::Down, false),
                        Some(Keycode::Left) => self.apad.set_button(butt::Left, false),
                        Some(Keycode::Right) => self.apad.set_button(butt::Right, false),
                        Some(Keycode::X) => self.apad.set_button(butt::A, false),
                        Some(Keycode::Z) => self.apad.set_button(butt::B, false),
                        Some(Keycode::A) => self.apad.set_button(butt::Y, false),
                        Some(Keycode::S) => self.apad.set_button(butt::X, false),
                        Some(Keycode::_1) => self.apad.set_button(butt::LeftTrigger, false),
                        Some(Keycode::Q) => self.apad.set_button(butt::LeftShoulder, false),
                        Some(Keycode::_2) => self.apad.set_button(butt::RightTrigger, false),
                        Some(Keycode::W) => self.apad.set_button(butt::RightShoulder, false),
                        Some(Keycode::Tab) => self.apad.set_button(butt::Select, false),
                        Some(Keycode::Return) => self.apad.set_button(butt::Start, false),
                        Some(Keycode::Escape) => self.apad.set_button(butt::Menu, false),
                        Some(_) => {} // ignore the rest
                    }
                }
                Event::Quit { .. } => self.quit_requested = true,
                _ => {}
            }
        }
        self.canvas.set_draw_color(self.bg_color);
        self.canvas.clear();
        self.canvas.set_draw_color((255, 255, 255, 255));
        Ok(())
    }

    /// Uses SDL's "texture.with_lock" function to access the pixel buffer as an RGB array.
    pub fn pixel_buffer_update<F, R>(&mut self, func: F) -> SdlResult<()>
    where
        F: FnOnce(&mut [u8], usize) -> R,
    {
        self.pixel_buffer.with_lock(None, func)?;
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
    pub fn pixel_buffer_present(&mut self) -> SdlResult<()> {
        let rect = self.get_scaled_rect().unwrap(); // TODO: Clean up unwrap
        self.canvas
            .copy_ex(&self.pixel_buffer, None, rect, 0.0, None, false, false)?;
        Ok(())
    }

    /// Presents the render target to the canvas respecting the scaling strategy.
    /// Warning: can be much slower than "pixel_buffer_present" if the goal is to simply
    /// draw pixel-by-pixel.
    pub fn render_target_present(&mut self) -> SdlResult<()> {
        let rect = self.get_scaled_rect().unwrap(); // TODO: Clean up unwrap
        self.canvas
            .copy_ex(&self.render_target, None, rect, 0.0, None, false, false)?;
        Ok(())
    }

    /// Required to be called at the end of a frame loop. Presents the canvas and performs an idle wait
    /// if frame rate limiting is required. Ironically, performing this idle loop may *lower* the CPU
    /// use in some platforms, compared to pure VSync!
    pub fn frame_finish(&mut self) -> SdlResult<()> {
        if self.app_time.elapsed().as_secs_f32() > 0.5 {
            // Skips the first frames
            self.update_time_buffer
                .push(self.frame_start.elapsed().as_secs_f64());
        }

        // Overlay
        #[cfg(feature = "ttf")]
        {
            if self.display_overlay {
                if let Some(font) = &mut self.default_font {
                    // self.canvas.set_draw_color((255, 255, 255, 255));
                    let mut y = self.overlay_coords.y;
                    for line in self.overlay.drain(..) {
                        font.draw(
                            line,
                            self.overlay_coords.x,
                            y,
                            self.overlay_scale,
                            &mut self.canvas,
                        )?;
                        let inc =
                            (font.height() as f32 * self.overlay_line_spacing) * self.overlay_scale;
                        y += (inc * font.line_spacing) as i32;
                    }
                }
            }
        }

        // TESTING: Moved here from end of function, right before the "Ok(())"
        self.canvas.present();

        match self.timing {
            // Optional FPS limiting
            Timing::VsyncLimitFPS(fps_limit) | Timing::ImmediateLimitFPS(fps_limit) => {
                // let update_so_far = self.frame_start.elapsed().as_secs_f64();
                // let target_time = (1.0 / fps_limit);// - 0.0001;
                // let diff = target_time - update_so_far;
                // if diff > 0.0 {
                //     std::thread::sleep(Duration::from_secs_f64(diff));
                // }

                const LARGE_STEP: f64 = 1.0 / 1000.0; // 1ms
                const SMALL_STEP: f64 = 1.0 / 10000.0; // 0.1ms
                let mut update_so_far = self.frame_start.elapsed().as_secs_f64();
                let target_time = 1.0 / fps_limit;
                // match self.timing {
                //     // Helps to ensure target_time ends just before vsync
                //     Timing::VsyncLimitFPS(_) => (1.0 / fps_limit) - 0.0001,
                //     _ => 1.0 / fps_limit,
                // };
                while update_so_far < target_time {
                    update_so_far = self.frame_start.elapsed().as_secs_f64();
                    let diff = target_time - update_so_far;
                    if diff > LARGE_STEP {
                        std::thread::sleep(Duration::from_secs_f64(LARGE_STEP));
                    } else if diff > SMALL_STEP {
                        std::thread::sleep(Duration::from_secs_f64(SMALL_STEP));
                    } else {
                        break;
                    }
                }
            }
            // Vsync or Immediate don't sleep
            _ => {}
        };

        // Detects new second, prints FPS
        if let Some(interval) = self.print_fps_interval {
            if self.last_second.elapsed().as_secs_f32() > interval {
                self.last_second = Instant::now();
                println!("FPS: {:.1}", (1.0 / self.elapsed_time));
            }
        }

        Ok(())
    }

    // Audio
    /// Initiates playback of audio device.
    pub fn audio_start(&mut self) -> SdlResult<()> {
        if let Some(audio) = &mut self.audio_stream {
            audio.resume()?;
            // audio.resume();
        }
        Ok(())
    }

    /// Pauses playback of audio device.
    pub fn audio_pause(&mut self) -> SdlResult<()> {
        if let Some(audio) = &mut self.audio_stream {
            audio.pause()?;
            // audio.pause();
        }
        Ok(())
    }

    /// Returns the current audio mix rate. TODO: This is locked at 44100Hz, should be user adjustable.
    pub fn audio_mixrate(&self) -> Option<u32> {
        self.sample_rate
    }

    /// Copies a slice of StereoFrames to the audio buffer. Ideally you should call this only once per frame,
    /// with all the samples that you need for that frame.
    pub fn audio_push_samples(&mut self, samples: &[i16]) -> SdlResult<()> {
        let Some(stream) = &mut self.audio_stream else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Audio device not found",
            )));
        };

        stream.put_data_i16(samples)?;
        Ok(())
    }

    /// Estimates how many stereo frames to fill the buffer at the current frame,
    /// with minimum lag and without audio cut-offs.
    pub fn audio_samples_per_frame(&self) -> Option<usize> {
        let sample_rate = self.sample_rate?;
        let queued_samples = (sample_rate as f64 * self.elapsed_time_raw) as usize;

        // Calculate ideal number of samples based on elapsed time
        let ideal_samples = (sample_rate as f64 * self.elapsed_time_raw) as usize;

        // Adjust for existing buffer content - if buffer has grown too large, reduce samples
        let buffer_target = (sample_rate as f64 * 0.02) as usize; // 20ms buffer

        if queued_samples > buffer_target {
            // Buffer growing too large, generate fewer samples
            let overflow = queued_samples - buffer_target;
            if overflow >= ideal_samples {
                None // Don't generate any samples this frame
            } else {
                Some(ideal_samples - overflow)
            }
        } else {
            // Buffer is small enough, generate normal amount
            Some(ideal_samples)
        }
    }
}

#[inline(always)]
// Skips quantization if value is too tiny, useful when getting elapsed time in
// immediate timing mode and very fast frame rates.
pub(crate) fn quantize(value: f64, size: f64) -> f64 {
    let result = (value / size).round() * size;
    if result < f64::EPSILON {
        value
    } else {
        result
    }
}

#[allow(unused)]
pub(crate) fn next_power_of_two(mut n: u32) -> u32 {
    if n.is_power_of_two() {
        return n;
    }
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n += 1;
    n
}

// pub(crate) fn prev_power_of_two(n: u32) -> u32 {
//     if n.is_power_of_two() {
//         return n;
//     }
//     let mut x = n;
//     x |= x >> 1;
//     x |= x >> 2;
//     x |= x >> 4;
//     x |= x >> 8;
//     x |= x >> 16;
//     (x >> 1) + 1
// }
