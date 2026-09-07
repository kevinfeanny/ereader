use anyhow::Result;
use image::{RgbaImage, Rgba};
use ab_glyph::{FontRef, PxScale, ScaleFont, point};
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;

// Display dimensions
pub const WIDTH:  u32 = 1024;
pub const HEIGHT: u32 = 600;

// Margins
pub const MARGIN_TOP:    u32 = 86;
pub const MARGIN_BOTTOM: u32 = 86;
pub const MARGIN_LEFT:   u32 = 94;
pub const MARGIN_RIGHT:  u32 = 94;
pub const FONT_SIZE:     f32 = 22.0;
pub const LINE_HEIGHT:   u32 = 28;
pub const STATUS_HEIGHT: u32 = 25;

const FRAMEBUFFER: &str = "/dev/fb0";
const BPP: u32 = 4; // 32 bits per pixel = 4 bytes

pub struct Display {
    font: Vec<u8>,
}

impl Display {
    pub fn new(font_data: Vec<u8>) -> Result<Self> {
        Ok(Display { font: font_data })
    }

    pub fn init(&mut self) -> Result<()> {
        self.clear()?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        let size = (WIDTH * HEIGHT * BPP) as usize;
        let blank = vec![0u8; size];
        self.write_framebuffer(&blank)
    }

    pub fn display_image(&mut self, image: &RgbaImage) -> Result<()> {
        let mut buffer = Vec::with_capacity((WIDTH * HEIGHT * BPP) as usize);

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let pixel = image.get_pixel(x, y);
                // BGRA format for Pi framebuffer
                buffer.push(pixel[3]); // A
                buffer.push(pixel[0]); // R
                buffer.push(pixel[1]); // G
                buffer.push(pixel[2]); // B
            }
        }

        self.write_framebuffer(&buffer)
    }

    fn write_framebuffer(&self, data: &[u8]) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_SYNC)
            .open(FRAMEBUFFER)?;
        file.write_all(data)?;
        Ok(())
    }

    pub fn render_page(
        &mut self,
        lines:       &[String],
        page_num:    usize,
        total_pages: usize,
        title:       &str,
        battery:     &str,
    ) -> Result<()> {
        let mut image = RgbaImage::from_pixel(WIDTH, HEIGHT, Rgba([255, 255, 255, 255]));

        let font = FontRef::try_from_slice(&self.font)
            .map_err(|e| anyhow::anyhow!("Font error: {:?}", e))?;

        let scale       = PxScale::from(FONT_SIZE);
        let scale_small = PxScale::from(16.0);

        // Draw text lines
        let mut y = MARGIN_TOP as f32;
        for line in lines {
            draw_text(&mut image, &font, scale, MARGIN_LEFT as f32, y, [0,0,0,255]);
            draw_text_str(&mut image, &font, scale, MARGIN_LEFT as f32, y, line);
            y += LINE_HEIGHT as f32;
        }

        // Status bar
        let status_y = HEIGHT - MARGIN_BOTTOM;
        draw_hline(&mut image, MARGIN_LEFT, status_y, WIDTH - MARGIN_RIGHT);

        let status = format!(
            "{}  |  Page {} of {}  |  {}",
            &title[..title.len().min(25)],
            page_num + 1,
            total_pages,
            battery
        );
        draw_text_str(&mut image, &font, scale_small,
                      MARGIN_LEFT as f32, (status_y + 4) as f32, &status);

        self.display_image(&image)
    }

    pub fn render_message(&mut self, message: &str) -> Result<()> {
        let mut image = RgbaImage::from_pixel(WIDTH, HEIGHT, Rgba([255, 255, 255, 255]));

        let font = FontRef::try_from_slice(&self.font)
            .map_err(|e| anyhow::anyhow!("Font error: {:?}", e))?;

        let scale = PxScale::from(FONT_SIZE);

        let mut y = MARGIN_TOP as f32;
        for line in message.lines() {
            draw_text_str(&mut image, &font, scale, MARGIN_LEFT as f32, y, line);
            y += LINE_HEIGHT as f32;
        }

        self.display_image(&image)
    }

    pub fn render_menu(
        &mut self,
        title:  &str,
        items:  &[(String, bool)],
        status: &str,
    ) -> Result<()> {
        let mut image = RgbaImage::from_pixel(WIDTH, HEIGHT, Rgba([255, 255, 255, 255]));

        let font = FontRef::try_from_slice(&self.font)
            .map_err(|e| anyhow::anyhow!("Font error: {:?}", e))?;

        let scale_title  = PxScale::from(28.0);
        let scale_item   = PxScale::from(FONT_SIZE);
        let scale_status = PxScale::from(14.0);

        // Title
        draw_text_str(&mut image, &font, scale_title,
                      MARGIN_LEFT as f32, MARGIN_TOP as f32, title);
        draw_hline(&mut image, MARGIN_LEFT, MARGIN_TOP + 36, WIDTH - MARGIN_RIGHT);

        // Menu items
        let mut y = (MARGIN_TOP + 46) as f32;
        let item_height = FONT_SIZE + 8.0;

        for (label, selected) in items {
            if *selected {
                fill_rect(&mut image, MARGIN_LEFT, y as u32 - 2,
                          WIDTH - MARGIN_RIGHT, y as u32 + FONT_SIZE as u32 + 4,
                          [0, 0, 0, 255]);
                draw_text_color(&mut image, &font, scale_item,
                                (MARGIN_LEFT + 8) as f32, y, label, [255,255,255,255]);
            } else {
                draw_text_color(&mut image, &font, scale_item,
                                (MARGIN_LEFT + 8) as f32, y, label, [0,0,0,255]);
            }
            y += item_height;
        }

        // Status bar
        let status_y = HEIGHT - MARGIN_BOTTOM;
        draw_hline(&mut image, MARGIN_LEFT, status_y, WIDTH - MARGIN_RIGHT);
        draw_text_color(&mut image, &font, scale_status,
                        MARGIN_LEFT as f32, (status_y + 4) as f32, status, [0,0,0,255]);

        self.display_image(&image)
    }

    pub fn render_numpad(
        &mut self,
        numpad: &[[&str; 3]; 4],
        row:    usize,
        col:    usize,
        entry:  &str,
    ) -> Result<()> {
        let mut image = RgbaImage::from_pixel(WIDTH, HEIGHT, Rgba([255, 255, 255, 255]));

        let font = FontRef::try_from_slice(&self.font)
            .map_err(|e| anyhow::anyhow!("Font error: {:?}", e))?;

        let scale       = PxScale::from(28.0);
        let scale_small = PxScale::from(16.0);
        let scale_large = PxScale::from(32.0);

        draw_text_color(&mut image, &font, scale_large,
                        MARGIN_LEFT as f32, MARGIN_TOP as f32,
                        "GO TO PAGE", [0,0,0,255]);
        draw_hline(&mut image, MARGIN_LEFT, MARGIN_TOP + 40, WIDTH - MARGIN_RIGHT);

        let display_entry = if entry.is_empty() { "_" } else { entry };
        let entry_text    = format!("Page: {}", display_entry);
        draw_text_color(&mut image, &font, scale_large,
                        MARGIN_LEFT as f32, (MARGIN_TOP + 50) as f32,
                        &entry_text, [0,0,0,255]);
        draw_hline(&mut image, MARGIN_LEFT, MARGIN_TOP + 90, WIDTH - MARGIN_RIGHT);

        let usable_width = WIDTH - MARGIN_LEFT - MARGIN_RIGHT;
        let cell_w = (usable_width / 3).min(100);
        let cell_h = 50u32;
        let grid_w = cell_w * 3;
        let start_x = MARGIN_LEFT + (usable_width - grid_w) / 2;
        let start_y = MARGIN_TOP + 105;

        for (row_idx, numpad_row) in numpad.iter().enumerate() {
            for (col_idx, key) in numpad_row.iter().enumerate() {
                let x = start_x + col_idx as u32 * cell_w;
                let y = start_y + row_idx as u32 * cell_h;

                if row_idx == row && col_idx == col {
                    fill_rect(&mut image, x, y, x + cell_w - 4, y + cell_h - 4, [0,0,0,255]);
                    draw_text_color(&mut image, &font, scale,
                                    (x + 15) as f32, (y + 10) as f32, key, [255,255,255,255]);
                } else {
                    draw_rect(&mut image, x, y, x + cell_w - 4, y + cell_h - 4);
                    draw_text_color(&mut image, &font, scale,
                                    (x + 15) as f32, (y + 10) as f32, key, [0,0,0,255]);
                }
            }
        }

        let instructions = "FWD=Right BACK=Left BK1=Up BK2=Down BK3=Select BK4=Cancel";
        draw_text_color(&mut image, &font, scale_small,
                        MARGIN_LEFT as f32, (HEIGHT - MARGIN_BOTTOM + 4) as f32,
                        instructions, [0,0,0,255]);

        self.display_image(&image)
    }

    pub fn sleep_display(&mut self) -> Result<()> {
        Ok(())
    }
}

// ── Drawing helpers ───────────────────────────────────────────────

fn draw_text_str(
    image: &mut RgbaImage,
    font:  &FontRef,
    scale: PxScale,
    x:     f32,
    y:     f32,
    text:  &str,
) {
    draw_text_color(image, font, scale, x, y, text, [0, 0, 0, 255]);
}

fn draw_text(
    image: &mut RgbaImage,
    font:  &FontRef,
    scale: PxScale,
    x:     f32,
    y:     f32,
    _color: [u8; 4],
) {
    let _ = (image, font, scale, x, y);
}

fn draw_text_color(
    image: &mut RgbaImage,
    font:  &FontRef,
    scale: PxScale,
    x:     f32,
    y:     f32,
    text:  &str,
    color: [u8; 4],
) {
    use ab_glyph::Font;
    let mut cursor_x = x;

    for ch in text.chars() {
        let glyph_id = font.glyph_id(ch);
        let glyph    = glyph_id.with_scale_and_position(scale, point(cursor_x, y + scale.y));

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, coverage| {
                let px = bounds.min.x as u32 + gx;
                let py = bounds.min.y as u32 + gy;
                if px < image.width() && py < image.height() {
                    let alpha = (coverage * 255.0) as u8;
                    let bg    = image.get_pixel(px, py);
                    let r = blend(bg[0], color[0], alpha);
                    let g = blend(bg[1], color[1], alpha);
                    let b = blend(bg[2], color[2], alpha);
                    image.put_pixel(px, py, Rgba([r, g, b, 255]));
                }
            });
        }

        cursor_x += font.as_scaled(scale).h_advance(glyph_id);
    }
}

fn blend(bg: u8, fg: u8, alpha: u8) -> u8 {
    let a = alpha as u32;
    let b = bg as u32;
    let f = fg as u32;
    ((f * a + b * (255 - a)) / 255) as u8
}

fn draw_hline(image: &mut RgbaImage, x1: u32, y: u32, x2: u32) {
    for x in x1..x2 {
        if x < image.width() && y < image.height() {
            image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
        }
    }
}

fn draw_rect(image: &mut RgbaImage, x1: u32, y1: u32, x2: u32, y2: u32) {
    for x in x1..=x2 {
        if x < image.width() {
            if y1 < image.height() { image.put_pixel(x, y1, Rgba([0,0,0,255])); }
            if y2 < image.height() { image.put_pixel(x, y2, Rgba([0,0,0,255])); }
        }
    }
    for y in y1..=y2 {
        if y < image.height() {
            if x1 < image.width() { image.put_pixel(x1, y, Rgba([0,0,0,255])); }
            if x2 < image.width() { image.put_pixel(x2, y, Rgba([0,0,0,255])); }
        }
    }
}

fn fill_rect(image: &mut RgbaImage, x1: u32, y1: u32, x2: u32, y2: u32, color: [u8; 4]) {
    for y in y1..=y2 {
        for x in x1..=x2 {
            if x < image.width() && y < image.height() {
                image.put_pixel(x, y, Rgba(color));
            }
        }
    }
}