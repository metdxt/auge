use super::{AugeFilter, FilterResult};
use crate::types::AugeError;

pub struct GBlurFilter {
    pub fast: bool,
    pub sigma: f32
}

impl AugeFilter for GBlurFilter {
    fn apply(&self, img: image::DynamicImage) -> Result<FilterResult, AugeError> {
        if self.fast {
            Ok(img.fast_blur(self.sigma).into())
        } else {
            Ok(img.blur(self.sigma).into())
        }
    }
}