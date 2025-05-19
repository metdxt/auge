use image::{DynamicImage, GrayImage, Luma};
use crate::types::AugeError;
use super::{FilterResult, AugeFilter};

pub struct EdgeFilter;

impl AugeFilter for EdgeFilter {
    fn apply(&self, img: DynamicImage) -> Result<FilterResult, AugeError> {
        let gray_img = img.to_luma8();
        let (width, height) = gray_img.dimensions();
        let mut edge_img = GrayImage::new(width, height);

        // Sobel kernels
        let sobel_x: [i32; 9] = [-1, 0, 1, -2, 0, 2, -1, 0, 1];
        let sobel_y: [i32; 9] = [-1, -2, -1, 0, 0, 0, 1, 2, 1];

        for y in 1..height-1 {
            for x in 1..width-1 {
                let mut gx = 0;
                let mut gy = 0;
                
                // Apply Sobel operator
                for ky in 0..3 {
                    for kx in 0..3 {
                        let pixel = gray_img.get_pixel(x + kx - 1, y + ky - 1)[0] as i32;
                        gx += pixel * sobel_x[(ky * 3 + kx) as usize];
                        gy += pixel * sobel_y[(ky * 3 + kx) as usize];
                    }
                }
                
                let magnitude = ((gx * gx + gy * gy) as f32).sqrt() as u8;
                edge_img.put_pixel(x, y, Luma([magnitude]));
            }
        }

        Ok(DynamicImage::ImageLuma8(edge_img).into())
    }
}
