use image::{DynamicImage, ImageBuffer, Rgb};

use super::{AugeFilter, FilterResult};
use crate::types::AugeError;

pub struct DynamicThresholdFilter {
    pub lower_percentile: f32,
    pub upper_percentile: f32,
    
    pub color_black: Rgb<u8>,
    pub color_mid: Rgb<u8>,
    pub color_white: Rgb<u8>
}

impl Default for DynamicThresholdFilter {
    fn default() -> Self {
        Self {
            lower_percentile: 0.2,
            upper_percentile: 0.2,
            color_black: Rgb::from([0u8; 3]),
            color_white: Rgb::from([255u8; 3]),
            color_mid: Rgb::from([127u8; 3])
        }
    }
}

impl AugeFilter for DynamicThresholdFilter {
    fn apply(&self, img: image::DynamicImage) -> Result<FilterResult, AugeError> {
        let luma_img = img.to_luma8();
        let (width, height) = luma_img.dimensions();
        let total_pixels = width as usize * height as usize; // Используем умножение

        if total_pixels == 0 {
             return Ok(FilterResult::Image(DynamicImage::ImageRgb8(
                 ImageBuffer::new(0, 0),
             )));
        }

        let mut histogram = [0u32; 256];
        for pixel in luma_img.pixels() {
            histogram[pixel[0] as usize] += 1;
        }

        let lower_cutoff_count = (total_pixels as f32 * self.lower_percentile).round() as u32;
        let lower_cutoff_count = lower_cutoff_count.min(total_pixels as u32);

        let upper_cutoff_target_count = (total_pixels as f32 * (1.0 - self.upper_percentile)).round() as u32;
        let upper_cutoff_target_count = upper_cutoff_target_count.min(total_pixels as u32);


        let mut cumulative_count = 0u32;
        let mut t_black = 0u8;
        let mut t_white = 255u8;

        let mut t_black_found = false;
        let mut t_white_found = false;

        for level in 0..=255 {
            let count_at_level = histogram[level];

            if !t_black_found && cumulative_count + count_at_level >= lower_cutoff_count {
                t_black = level as u8;
                t_black_found = true;
            }

            if !t_white_found && cumulative_count + count_at_level >= upper_cutoff_target_count {
                t_white = level as u8;
                t_white_found = true;
            }

            cumulative_count += count_at_level;

            if t_black_found && t_white_found {
                 break;
            }
        }
        if t_white <= t_black {
            if t_black > 0 {
                t_black -= 1;
            } else if t_white < 255 {
                 t_white += 1;
            }
        }

        let mut output_img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(width, height);
        for (x, y, luma_pixel) in luma_img.enumerate_pixels() {
            let luma_value = luma_pixel[0];
            let output_pixel = output_img.get_pixel_mut(x, y);

            if luma_value <= t_black {
                *output_pixel = self.color_black;
            } else if luma_value >= t_white {
                *output_pixel = self.color_white;
            } else {
                *output_pixel = self.color_mid;
            }
        }

        Ok(FilterResult::Image(DynamicImage::ImageRgb8(output_img)))
    }
}