use super::{AugeFilter, FilterResult};
use crate::types::AugeError;

pub struct InvertFilter;

impl AugeFilter for InvertFilter {
    fn apply(&self, img: image::DynamicImage) -> Result<FilterResult, AugeError> {
        let mut img = img;
        img.invert();
        Ok(img.into())
    }
}
