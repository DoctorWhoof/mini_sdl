mod gamepad;
mod scaling;
mod timing;
pub use gamepad::{Button, GamePad};
pub use scaling::Scaling;
pub use timing::Timing;

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    rect::Rect,
    render::{Canvas, Texture, TextureAccess},
    video::Window,
    EventPump,
};
use std::time::{Duration, Instant};

pub struct App {
    // State
    pub quit_requested: bool,
    pub gamepad: GamePad,
    request_update: bool,

    // Video
    width: u32,
    height: u32,
    dpi_mult: f32,
    timing: Timing,
    scaling: Scaling,

    // Timing,
    partial_time: Instant,
    frame_instant: Instant,
    update_start: Instant,
    update_time: f64,  // Time the last update took
    elapsed_time: f64, // Whole frame time at current FPS

    // SDL
    event_pump: EventPump,
    canvas: Canvas<Window>,
    texture: Texture,
}

impl App {
    pub fn new(width: u32, height: u32, timing: Timing, scaling: Scaling) -> Result<Self, String> {
        // SDL Init
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window("sdl spy", width * 2, height * 2)
            .allow_highdpi()
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = match timing {
            Timing::Vsync | Timing::VsyncLimitFPS(_) => window
                .into_canvas()
                .accelerated()
                .present_vsync()
                .build()
                .map_err(|e| e.to_string())?,
            Timing::Immediate | Timing::ImmediateLimitFPS(_) => window
                .into_canvas()
                .accelerated()
                .build()
                .map_err(|e| e.to_string())?,
        };

        use sdl2::sys::SDL_WindowFlags::*;
        let dpi_mult = if (canvas.window().window_flags() & SDL_WINDOW_ALLOW_HIGHDPI as u32) != 0 {
            2.0
        } else {
            1.0
        };

        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture(
                Some(PixelFormatEnum::RGB24),
                TextureAccess::Streaming,
                width as u32,
                height as u32,
            )
            .ok()
            .unwrap();

        let event_pump = sdl_context.event_pump()?;

        Ok(Self {
            quit_requested: false,
            request_update: true,
            gamepad: GamePad::new(),
            partial_time: Instant::now(),
            frame_instant: Instant::now(),
            update_start: Instant::now(),
            update_time: 0.0,
            elapsed_time: 0.0,
            width,
            height,
            dpi_mult,
            timing,
            scaling,
            canvas,
            texture,
            event_pump,
        })
    }

    pub fn update_requested(&self) -> bool {
        self.request_update
    }

    pub fn start_frame(&mut self) {
        // Whole frame time
        self.elapsed_time = self.frame_instant.elapsed().as_secs_f64();
        self.frame_instant = Instant::now();

        // Optional Frame skip
        self.request_update = match self.timing {
            Timing::VsyncLimitFPS(fps) | Timing::ImmediateLimitFPS(fps) => {
                let mut idle_time = (1.0 / fps) - self.update_time;
                if self.timing == Timing::VsyncLimitFPS(fps) {
                    idle_time *= 0.9 // ensures update happens before next vsync
                };
                let elapsed = self.partial_time.elapsed().as_secs_f64();
                if elapsed > idle_time {
                    // if (elapsed - idle_time).abs() > 0.001 {
                    // println!(
                    //     "update:{:.2} ms, idle:{:.2} ms, elapsed:{:.2} ms, frame: {:.2} ms",
                    //     update_time * 1000.0,
                    //     idle_time * 1000.0,
                    //     elapsed * 1000.0,
                    //     frame_time * 1000.0
                    // );
                }
                self.partial_time = Instant::now();
                true
            }
            _ => true,
        };
        self.update_start = Instant::now();
    }

    pub fn process_gamepad(&mut self) {
        self.gamepad.set_previous_state();
        for event in self.event_pump.poll_iter() {
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
                        Some(_) => {} // ignore the rest
                    }
                }
                _ => {}
            }
        }
    }

    pub fn update_pixel_buffer<F, R>(&mut self, func: F)
    where
        F: FnOnce(&mut [u8], usize) -> R,
    {
        if !self.request_update {
            return;
        }
        if let Err(text) = self.texture.with_lock(None, func) {
            println!("Error updating canvas texture: {}", text);
        }
    }

    pub fn finish_frame(&mut self) {
        if self.request_update {
            // Scaling math
            let rect = match self.scaling {
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
            };

            self.canvas.clear();

            self.canvas
                .copy_ex(&self.texture, None, rect, 0.0, None, false, false)
                .unwrap();

            self.update_time = self
                .update_start
                .elapsed()
                .as_secs_f64()
                .clamp(0.0, 1.0 / 30.0);

            self.canvas.present();
        } else {
            // Minimum sleep. Render and Input aren't processed while frame is waiting
            std::thread::sleep(Duration::from_micros(100));
        }
    }
}
