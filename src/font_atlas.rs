use crate::{next_power_of_two, SdlResult};
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::{Canvas, ScaleMode, Texture, TextureCreator},
    surface::Surface,
    sys::pixels::SDL_PixelFormat,
    ttf::Sdl3TtfContext,
    video::{Window, WindowContext},
};
use std::{collections::HashMap, path::Path};

const CHARACTERS: &'static str =
    "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!@#$%^&*()-_=+,.:;'~? ";

/// Pre-renders ASCII characters to a texture when created
/// and can render lines of text to a canvas. Does not handle Unicode characters!
pub struct FontAtlas {
    pub texture: Texture,
    pub color: Color,
    height: f32,
    rects: HashMap<char, Rect>,
}

impl FontAtlas {
    pub fn new(
        path: impl AsRef<Path>,
        size: f32,
        ttf: &Sdl3TtfContext,
        texture_creator: &mut TextureCreator<WindowContext>,
    ) -> SdlResult<Self> {
        let ttf_font = ttf.load_font(path, size).map_err(|e| e.to_string())?;

        // Obtain character metrics, populate rects and chars vectors in the same order.
        let mut char_rects = vec![];
        let mut chars = vec![];
        let mut x: i32 = 0;
        for ch in CHARACTERS.chars() {
            let Some(m) = ttf_font.find_glyph_metrics(ch) else {
                continue;
            };
            let w = m.advance as u32; // + m.minx.abs() as u32; // Removed since it was breaking a mono-spaced font, needs more testing
            let h = ttf_font.height() - m.miny.abs();
            let y = m.miny.abs();
            let rect = Rect::new(x, y, w, h as u32);
            char_rects.push(rect);
            chars.push(ch);
            x += m.advance;
            // println!("{}: {:?}, {:?}", ch, rect, m);
        }

        // Render individual surfaces for each character
        let mut pixel_count = 0;
        let mut surfaces = vec![];
        for ch in CHARACTERS.chars() {
            let surf = ttf_font
                .render_char(ch)
                .blended(Color::RGBA(255, 255, 255, 255))
                .map_err(|e| e.to_string())?;
            pixel_count += surf.width() * surf.height();
            surfaces.push(surf);
        }

        // Combine all character surfaces into a single atlas surface
        let res = next_power_of_two((pixel_count as f32).sqrt().ceil() as u32);
        let mut atlas = Surface::new(res, res, unsafe {
            PixelFormat::from_ll(SDL_PixelFormat::RGBA32)
        })?;
        let mut rects = HashMap::new();
        let mut row_height = 0;
        let mut x = 0;
        let mut y = 0;
        for (i, surf) in &mut surfaces.iter().enumerate() {
            if x + surf.width() as i32 >= res as i32 {
                x = 0;
                y += row_height;
                row_height = 0;
            }
            let Some(dst_rect) = char_rects.get(i) else {
                continue;
            };
            let new_rect = Rect::new(
                x,
                y,
                dst_rect.width(),
                dst_rect.height() + dst_rect.y as u32,
            );
            surf.blit(None, &mut atlas, new_rect)?;
            row_height = row_height.max(surf.height() as i32);
            x += surf.width() as i32;
            rects.insert(chars[i], new_rect);
        }

        // Generate texture from surface
        let mut texture = texture_creator.create_texture_from_surface(&atlas)?;
        // texture.set_scale_mode(ScaleMode::Nearest);

        // Finish
        Ok(Self {
            texture,
            rects,
            height: size,
            color: Color::WHITE,
        })
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn draw(
        &mut self,
        text: impl Into<String>,
        x: i32,
        y: i32,
        scale: f32,
        canvas: &mut Canvas<Window>,
    ) -> SdlResult<()> {
        let text: String = text.into();
        let mut x = x;

        self.texture
            .set_color_mod(self.color.r, self.color.g, self.color.b);

        for ch in text.chars() {
            if let Some(rect) = self.rects.get(&ch) {
                let dest = Rect::new(
                    x,
                    y,
                    (rect.width() as f32 * scale.abs()) as u32,
                    (rect.height() as f32 * scale.abs()) as u32,
                );
                canvas.copy(&self.texture, *rect, dest)?;
                x += rect.w * scale as i32;
            }
        }
        Ok(())
    }
}
