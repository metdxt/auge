use image::{DynamicImage, Rgb};
use crate::types::AugeError;
use super::{FilterResult, AugeFilter};

pub struct SepiaFilter;

impl AugeFilter for SepiaFilter {
    fn apply(&self, img: DynamicImage) -> Result<FilterResult, AugeError> {
        let mut img = img.to_rgb8();
        
        for pixel in img.pixels_mut() {
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;
            
            // Apply sepia tone transformation
            let new_r = (r * 0.393 + g * 0.769 + b * 0.189).min(255.0) as u8;
            let new_g = (r * 0.349 + g * 0.686 + b * 0.168).min(255.0) as u8;
            let new_b = (r * 0.272 + g * 0.534 + b * 0.131).min(255.0) as u8;
            
            *pixel = Rgb([new_r, new_g, new_b]);
        }
        
        Ok(DynamicImage::ImageRgb8(img).into())
    }
}
