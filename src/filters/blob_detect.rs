use crate::filters::{AugeFilter, FilterResult};
use crate::types::AugeError;
use clap::ValueEnum;
use image::{DynamicImage, GenericImageView, Pixel, Rgb, RgbImage, Rgba, RgbaImage};
use std::collections::VecDeque;

#[derive(Debug, Clone, ValueEnum)]
pub enum BlobColorMode {
    Rainbow,
    Heatmap,
    Solid,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum BlobBackground {
    Black,
    Transparent,
    Original,
}

pub struct BlobDetectFilter {
    pub threshold: u8,
    pub target_color: Option<Rgb<u8>>,
    pub mode: BlobColorMode,
    pub background: BlobBackground,
}

impl AugeFilter for BlobDetectFilter {
    fn apply(&self, img: DynamicImage) -> Result<FilterResult, AugeError> {
        let width = img.width();
        let height = img.height();

        if width == 0 || height == 0 {
            return Ok(FilterResult::Image(DynamicImage::new_rgba8(0, 0)));
        }

        let blobs = find_blobs(&img, self.threshold, self.target_color);
        let max_blob_size = blobs.iter().map(|b| b.len()).max().unwrap_or(0);

        let output_image = match self.background {
            BlobBackground::Black => {
                let mut canvas = RgbImage::new(width, height);
                // Background is already black (0,0,0) by default
                draw_blobs(&mut canvas, &blobs, &self.mode, max_blob_size);
                DynamicImage::ImageRgb8(canvas)
            }
            BlobBackground::Transparent => {
                let mut canvas = RgbaImage::new(width, height);
                // Background is 0,0,0,0 by default
                draw_blobs(&mut canvas, &blobs, &self.mode, max_blob_size);
                DynamicImage::ImageRgba8(canvas)
            }
            BlobBackground::Original => {
                let mut canvas = img.to_rgba8();
                draw_blobs(&mut canvas, &blobs, &self.mode, max_blob_size);
                DynamicImage::ImageRgba8(canvas)
            }
        };

        Ok(FilterResult::Image(output_image))
    }
}

fn find_blobs(
    img: &DynamicImage,
    threshold: u8,
    target_color: Option<Rgb<u8>>,
) -> Vec<Vec<(u32, u32)>> {
    let width = img.width();
    let height = img.height();

    if width == 0 || height == 0 {
        return vec![];
    }

    let mut queue: VecDeque<(u32, u32)> = VecDeque::new();
    queue.push_front((0, 0));

    let mut active_blob: Option<Vec<(u32, u32)>> = None;
    let mut blobs = vec![];
    let mut visited: Vec<bool> = vec![false; (width * height) as usize];

    const OFFSETS: [(i32, i32); 8] = [
        (1, 1),
        (0, 1),
        (1, 0),
        (-1, -1),
        (0, -1),
        (-1, 0),
        (1, -1),
        (-1, 1),
    ];

    while let Some(target_position) = queue.pop_front() {
        let (x, y) = target_position;
        let idx = (y * width + x) as usize;

        if visited[idx] {
            continue;
        }
        visited[idx] = true;

        let px = img.get_pixel(x, y);
        let matches = if let Some(target) = target_color {
            let p_rgb = px.to_rgb();
            let dr = p_rgb.0[0] as f32 - target.0[0] as f32;
            let dg = p_rgb.0[1] as f32 - target.0[1] as f32;
            let db = p_rgb.0[2] as f32 - target.0[2] as f32;
            (dr * dr + dg * dg + db * db).sqrt() <= threshold as f32
        } else {
            px.to_luma().0[0] <= threshold
        };

        if matches {
            match active_blob {
                Some(ref mut blob) => {
                    blob.push(target_position);
                }
                None => active_blob = Some(vec![target_position]),
            }
        } else if let Some(blob) = active_blob {
            blobs.push(blob);
            active_blob = None
        }

        for &(dx, dy) in &OFFSETS {
            let nx_i32 = x as i32 + dx;
            let ny_i32 = y as i32 + dy;

            if nx_i32 < 0 || nx_i32 as u32 >= width || ny_i32 < 0 || ny_i32 as u32 >= height {
                continue;
            }

            let nx = nx_i32 as u32;
            let ny = ny_i32 as u32;
            let n_idx = (ny * width + nx) as usize;

            if visited[n_idx] {
                continue;
            }

            let n_px = img.get_pixel(nx, ny);
            let matches = if let Some(target) = target_color {
                let p_rgb = n_px.to_rgb();
                let dr = p_rgb.0[0] as f32 - target.0[0] as f32;
                let dg = p_rgb.0[1] as f32 - target.0[1] as f32;
                let db = p_rgb.0[2] as f32 - target.0[2] as f32;
                (dr * dr + dg * dg + db * db).sqrt() <= threshold as f32
            } else {
                n_px.to_luma().0[0] <= threshold
            };

            if matches {
                queue.push_front((nx, ny));
            } else {
                queue.push_back((nx, ny));
            }
        }
    }

    if let Some(blob) = active_blob {
        blobs.push(blob);
    }

    blobs
}

trait PixelCanvas {
    fn put_pixel_rgba(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8);
}

impl PixelCanvas for RgbImage {
    fn put_pixel_rgba(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, _a: u8) {
        self.put_pixel(x, y, Rgb([r, g, b]));
    }
}

impl PixelCanvas for RgbaImage {
    fn put_pixel_rgba(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        self.put_pixel(x, y, Rgba([r, g, b, a]));
    }
}

fn draw_blobs<C: PixelCanvas>(
    canvas: &mut C,
    blobs: &[Vec<(u32, u32)>],
    mode: &BlobColorMode,
    max_blob_size: usize,
) {
    for (i, blob) in blobs.iter().enumerate() {
        let (r, g, b) = match mode {
            BlobColorMode::Rainbow => {
                let r = ((i * 100 + 50) % 255) as u8;
                let g = ((i * 50 + 100) % 255) as u8;
                let b = ((i * 20 + 150) % 255) as u8;
                (r, g, b)
            }
            BlobColorMode::Heatmap => {
                let size = blob.len();
                if max_blob_size == 0 {
                    (0, 0, 255)
                } else {
                    let t = size as f32 / max_blob_size as f32;
                    // Cool (Blue) -> Hot (Red) -> White
                    if t < 0.8 {
                        // Blue (0,0,255) to Red (255,0,0)
                        let ratio = t / 0.8;
                        let r_val = (255.0 * ratio) as u8;
                        let b_val = (255.0 * (1.0 - ratio)) as u8;
                        (r_val, 0, b_val)
                    } else {
                        // Red (255,0,0) to White (255,255,255)
                        let ratio = (t - 0.8) / 0.2;
                        let gb_val = (255.0 * ratio) as u8;
                        (255, gb_val, gb_val)
                    }
                }
            }
            BlobColorMode::Solid => (0, 255, 0), // Green
        };

        for &(x, y) in blob {
            canvas.put_pixel_rgba(x, y, r, g, b, 255);
        }
    }
}
