use std::{num::{ParseIntError, ParseFloatError}, str::FromStr};

use clap::ValueEnum;
use image::Rgb;
use serde::Serialize;

#[derive(thiserror::Error, Debug)]
pub enum AugeError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Image(#[from] image::ImageError),
    #[error("{0}")]
    Viu(#[from] viuer::ViuError),
    #[error("Integer Parse Error: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("Float Parse Error: {0}")]
    ParseFloat(#[from] ParseFloatError),
    #[error("Invalid color format, expected one of 'preserve', '#RRGGBB', or 'RRGGBB' got {0}")]
    InvalidColorFormat(String),
    #[error("Invalid resize format: {0}. Expected 'NN%', 'WIDTHxHEIGHT', 'autoxHEIGHT', 'WIDTHxauto', or 'autoxauto'.")] // Added for ResizeInput format errors
    InvalidResizeFormat(String),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, ValueEnum)]
pub enum EncodableFormats {
    Bmp,
    Farbfeld,
    Hdr,
    Ico,
    Jpeg,
    Exr,
    Png,
    Pnm,
    Qoi,
    Tga,
    Tiff,
    Webp,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputKind {
    Raster,
    Json,
}

#[derive(Debug, Clone)]
pub struct Color(pub Rgb<u8>);

impl FromStr for Color {
    type Err = AugeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex_code = s.to_lowercase();
        let hex_code = hex_code
            .strip_prefix('#')
            .unwrap_or(hex_code.as_str());
        if hex_code.len() == 6 {
            let r = u8::from_str_radix(&hex_code[0..2], 16)?;
            let g = u8::from_str_radix(&hex_code[2..4], 16)?;
            let b = u8::from_str_radix(&hex_code[4..6], 16)?;
            Ok(Color(Rgb([r, g, b])))
        } else {
            Err(AugeError::InvalidColorFormat(s.to_string()))
        }
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Rgb([r, g, b]) = self.0;
        // Format as 6-digit lowercase hexadecimal string with leading #
        let hex_string = format!("#{:02x}{:02x}{:02x}", r, g, b);
        serializer.serialize_str(&hex_string)
    }
}

#[derive(Debug, Clone)]
pub enum DotColorSource {
    Fixed(Rgb<u8>),
    Preserve,
}

impl FromStr for DotColorSource {
    type Err = AugeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "preserve" | "original" => Ok(DotColorSource::Preserve),
            hex_code => {
                let hex_code = hex_code.strip_prefix('#').unwrap_or(hex_code);
                if hex_code.len() == 6 {
                    let r = u8::from_str_radix(&hex_code[0..2], 16)?;
                    let g = u8::from_str_radix(&hex_code[2..4], 16)?;
                    let b = u8::from_str_radix(&hex_code[4..6], 16)?;
                    Ok(DotColorSource::Fixed(Rgb([r, g, b])))
                } else {
                    Err(AugeError::InvalidColorFormat(s.to_string()))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Dot {
    pub pos: (u32, u32),
    pub radius: i32,
    pub color: Color
}

#[derive(Debug, Clone, Serialize)]
pub struct DotFilterJson {
    pub bg: Color,
    pub points: Vec<Dot>
}

#[derive(Debug, Clone, PartialEq)]
pub enum AutoValue<T> where T: FromStr {
    Auto,
    Concrete(T)
}

impl<T> FromStr for AutoValue<T>
where
    T: FromStr,
    <T as FromStr>::Err: Into<AugeError>,
{
    type Err = AugeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("auto") {
            Ok(Self::Auto)
        } else {
            T::from_str(s)
                .map(Self::Concrete)
                .map_err(|err| err.into())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResizeInput {
    Relative(f32),
    Absolute(AutoValue<u32>, AutoValue<u32>)
}


impl FromStr for ResizeInput {
    type Err = AugeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Some(num_str) = s.strip_suffix('%') {
            let percentage = num_str.trim().parse::<f32>()?;
            if percentage < 0.0 {
                return Err(AugeError::InvalidResizeFormat(format!("Percentage cannot be negative: {}", s)));
            }
            return Ok(ResizeInput::Relative(percentage / 100.0));
        }

        let parts: Vec<&str> = s.splitn(2, 'x').collect();
        if parts.len() == 2 {
            let width_str = parts[0].trim();
            let height_str = parts[1].trim();

            let width_av = if width_str.is_empty() {
                Ok(AutoValue::Auto)
            } else {
                width_str.parse::<AutoValue<u32>>()
            }?;

            let height_av = if height_str.is_empty() {
                Ok(AutoValue::Auto)
            } else {
                height_str.parse::<AutoValue<u32>>()
            }?;

            return Ok(ResizeInput::Absolute(width_av, height_av));
        }

        Err(AugeError::InvalidResizeFormat(s.to_string()))
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum FilterType {
    /// Nearest Neighbor
    Nearest,

    /// Linear Filter
    Triangle,

    /// Cubic Filter
    CatmullRom,

    /// Gaussian Filter
    Gaussian,

    /// Lanczos with window 3
    Lanczos3,
}

impl Into<image::imageops::FilterType> for FilterType {
    fn into(self) -> image::imageops::FilterType {
        match self {
            Self::Nearest => image::imageops::FilterType::Nearest,
            Self::Triangle => image::imageops::FilterType::Triangle,
            Self::CatmullRom => image::imageops::FilterType::CatmullRom,
            Self::Gaussian => image::imageops::FilterType::Gaussian,
            Self::Lanczos3 => image::imageops::FilterType::Lanczos3,
        }
    }
}
