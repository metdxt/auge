use image::GenericImageView;

use super::{AugeFilter, FilterResult};
use crate::types::{AugeError, AutoValue, ResizeInput};


pub struct ResizeFilter {
    pub target: ResizeInput,
    pub exact: bool,
    pub filter: image::imageops::FilterType
}

impl AugeFilter for ResizeFilter {
    fn apply(&self, img: image::DynamicImage) -> Result<FilterResult, AugeError> {
        let (ox, oy) = img.dimensions();
        let (tx, ty) = match self.target {
            ResizeInput::Relative(multiplier) => {
                ((ox as f32 * multiplier).round() as u32, (oy as f32 * multiplier).round() as u32)
            }
            ResizeInput::Absolute(AutoValue::Auto, AutoValue::Auto) => (ox, oy),
            ResizeInput::Absolute(AutoValue::Concrete(tx), AutoValue::Concrete(ty)) => (tx, ty),
            ResizeInput::Absolute(AutoValue::Auto, AutoValue::Concrete(ty)) => {
                let multiplier = ty as f32 / oy as f32;
                ((ox as f32 * multiplier).round() as u32, ty)
            },
            ResizeInput::Absolute(AutoValue::Concrete(tx), AutoValue::Auto) => {
                let multiplier = tx as f32 / ox as f32;
                (tx, (oy as f32 * multiplier).round() as u32)
            }
        };

        if self.exact {
            Ok(img.resize_exact(tx, ty, self.filter).into())
        } else {
            Ok(img.resize(tx, ty, self.filter).into())
        }
        
    }
}
