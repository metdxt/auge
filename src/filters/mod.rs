pub mod dynthres;
pub mod gblur;
pub mod grayscale;
pub mod dotart;
pub mod resize;
pub mod invert;
pub mod sepia;

use dotart::DotartFilter;
use image::{DynamicImage, Rgb};
use invert::InvertFilter;
use sepia::SepiaFilter;

use crate::{Command, types::{AugeError, Color}};

pub enum FilterResult {
    Image(DynamicImage),
    Text(String),
}

impl From<DynamicImage> for FilterResult {
    fn from(value: DynamicImage) -> Self {
        Self::Image(value)
    }
}

pub trait AugeFilter {
    fn apply(&self, img: DynamicImage) -> Result<FilterResult, AugeError>;
}

pub struct NoOpFilter;

impl AugeFilter for NoOpFilter {
    fn apply(&self, img: DynamicImage) -> Result<FilterResult, AugeError> {
        Ok(img.into())
    }
}

pub fn filter_from_command(cmd: Command) -> Result<Box<dyn AugeFilter>, AugeError> {
    match cmd {
        Command::View => Ok(Box::new(NoOpFilter)),
        Command::Grayscale => Ok(Box::new(grayscale::GrayscaleFilter)),
        Command::GBlur { sigma, fast } => Ok(Box::new(gblur::GBlurFilter { sigma, fast })),
        Command::Dotart {
            output,
            scale,
            lower_percentile,
            upper_percentile,
            dot_color,
            bg_color
        } => Ok(Box::new(DotartFilter { output, scale, lower_percentile, upper_percentile, dot_color, bg_color })),
        Command::Dynthres {
            lower_percentile,
            upper_percentile,
            dark_color, mid_color, bright_color
        } => Ok(Box::new(dynthres::DynamicThresholdFilter {
            lower_percentile,
            upper_percentile,
            color_black: dark_color.unwrap_or(Color(Rgb::from([0u8; 3]))).0,
            color_white: bright_color.unwrap_or(Color(Rgb::from([255u8; 3]))).0,
            color_mid: mid_color.unwrap_or(Color(Rgb::from([127u8; 3]))).0,
        })),
        Command::Resize { target , exact, filter} => Ok(Box::new(resize::ResizeFilter { target,  exact, filter: filter.into() })),
        Command::Sepia => Ok(Box::new(SepiaFilter)),
    }
}
