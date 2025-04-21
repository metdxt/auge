use image::imageops;
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb, Rgba};
use imageproc::drawing::draw_filled_circle_mut;

use super::dynthres::DynamicThresholdFilter;
use super::{AugeFilter, FilterResult};
use crate::types::{AugeError, Color, DotColorSource, OutputKind, Dot, DotFilterJson};

pub struct DotartFilter {
    pub scale: u32,
    pub output: OutputKind,
    pub lower_percentile: f32,
    pub upper_percentile: f32,
    pub dot_color: DotColorSource,
    pub bg_color: Color,
}

impl Default for DotartFilter {
    fn default() -> Self {
        Self {
            scale: 8,
            output: OutputKind::Raster,
            lower_percentile: 0.75,
            upper_percentile: 0.10,
            dot_color: DotColorSource::Preserve,
            bg_color: Color(Rgb::from([0u8; 3])),
        }
    }
}

impl AugeFilter for DotartFilter {
    fn apply(&self, img: DynamicImage) -> Result<FilterResult, AugeError> {
        let scale = self.scale.max(1);
        let (width, height) = img.dimensions();

        let scaled_width = (width / scale).max(1);
        let scaled_height = (height / scale).max(1);

        let small_img = img.resize_exact(
            scaled_width,
            scaled_height,
            image::imageops::FilterType::Gaussian,
        );
        let small_luma_img = small_img.to_luma8();
        let small_dyn_luma_img = DynamicImage::ImageLuma8(small_luma_img);

        let filter = DynamicThresholdFilter {
            lower_percentile: self.lower_percentile,
            upper_percentile: self.upper_percentile,
            ..Default::default()
        };
        let threshold_result = filter.apply(small_dyn_luma_img)?;
        let small_threshold_rgb_buffer = match threshold_result {
            FilterResult::Image(DynamicImage::ImageRgb8(buffer)) => buffer,
            FilterResult::Image(other) => other.to_rgb8(),
            FilterResult::Text(_) => unreachable!(),
        };

        match self.output {
            OutputKind::Raster => {
                let mut background_layer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(
                    width,
                    height,
                    Rgba::from([
                        self.bg_color.0[0],
                        self.bg_color.0[1],
                        self.bg_color.0[2],
                        255u8,
                    ]),
                );
                let mut foreground_layer =
                    ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(width, height, Rgba([0, 0, 0, 0]));

                let radius = (scale / 4) as i32;
                let radius = radius.max(1);

                for (x, y, threshold_pixel) in small_threshold_rgb_buffer.enumerate_pixels() {
                    let threshold_luma = threshold_pixel[0];
                    if threshold_luma == 0 {
                        continue;
                    }

                    let center_x = (x * scale + scale / 2) as i32;
                    let center_y = (y * scale + scale / 2) as i32;

                    let rgb_part: Rgb<u8> = match &self.dot_color {
                        DotColorSource::Fixed(fixed_color) => *fixed_color,
                        DotColorSource::Preserve => small_img.get_pixel(x, y).to_rgb(),
                    };

                    let alpha = threshold_luma;
                    let circle_rgba_color = Rgba([rgb_part[0], rgb_part[1], rgb_part[2], alpha]);

                    draw_filled_circle_mut(
                        &mut foreground_layer,
                        (center_x, center_y),
                        radius,
                        circle_rgba_color,
                    );
                }

                imageops::overlay(&mut background_layer, &foreground_layer, 0, 0);

                Ok(FilterResult::Image(DynamicImage::ImageRgba8(
                    background_layer,
                )))
            }
            OutputKind::Json => {
                let radius = (scale / 4) as i32;
                let radius = radius.max(1);
                let mut dots = Vec::new();

                for (x, y, threshold_pixel) in small_threshold_rgb_buffer.enumerate_pixels() {
                    let threshold_luma = threshold_pixel[0];
                    if threshold_luma == 0 {
                        continue;
                    }

                    let center_x = (x * scale + scale / 2) as u32;
                    let center_y = (y * scale + scale / 2) as u32;

                    let rgb_part: Rgb<u8> = match &self.dot_color {
                        DotColorSource::Fixed(fixed_color) => *fixed_color,
                        DotColorSource::Preserve => small_img.get_pixel(x, y).to_rgb(),
                    };

                    dots.push(Dot {
                        pos: (center_x, center_y),
                        radius,
                        color: Color(rgb_part),
                    });
                }

                let json_data = DotFilterJson {
                    bg: self.bg_color.clone(),
                    points: dots,
                };
                let json_string = serde_json::to_string(&json_data)?;
                Ok(FilterResult::Text(json_string))
            }
        }
    }
}