use super::{AugeFilter, FilterResult};
use crate::types::AugeError;

pub struct GrayscaleFilter;

impl AugeFilter for GrayscaleFilter {
    fn apply(&self, img: image::DynamicImage) -> Result<FilterResult, AugeError> {
        Ok(img.grayscale().into())
    }
}
