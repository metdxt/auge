use std::io::{stdout, BufWriter, IsTerminal, Write, Cursor};

use image::{
    codecs::{
        bmp::BmpEncoder,
        farbfeld::FarbfeldEncoder,
        hdr::HdrEncoder,
        ico::IcoEncoder,
        jpeg::JpegEncoder,
        openexr::OpenExrEncoder,
        png::PngEncoder,
        pnm::PnmEncoder,
        qoi::QoiEncoder,
        tga::TgaEncoder,
        tiff::TiffEncoder,
        webp::WebPEncoder,
    },
    DynamicImage, ImageEncoder,
};
use viuer::{print, Config};

use crate::types::{AugeError, EncodableFormats};


/// This function outputs image to terminal, or writes into pipe in a specified format
pub fn print_image(img: &DynamicImage, format: EncodableFormats) -> Result<(), AugeError> {
    if stdout().is_terminal() {
        print(&img, &Config::default())?;
    } else {
        let stdout_handle = stdout().lock();
        let mut writer = BufWriter::new(stdout_handle);
        
        let pixels = img.as_bytes();
        let color_type = img.color();
        let (width, height) = (img.width(), img.height());
        
        match format {
            EncodableFormats::Bmp => {
                let encoder = BmpEncoder::new(&mut writer);
                encoder.write_image(pixels, width, height, color_type.into())?;
            }
            EncodableFormats::Farbfeld => {
                let encoder = FarbfeldEncoder::new(writer);
                encoder.write_image(pixels, width, height, color_type.into())?;
            }
            EncodableFormats::Hdr => {
                let encoder = HdrEncoder::new(writer);
                encoder.write_image(pixels, width, height, img.color().into())?;
            }
            EncodableFormats::Ico => {
                let encoder = IcoEncoder::new(writer);
                encoder.write_image(pixels, width, height, color_type.into())?;
            }
            EncodableFormats::Jpeg => {
                let encoder = JpegEncoder::new(writer);
                encoder.write_image(pixels, width, height, color_type.into())?;
            }
            EncodableFormats::Png => {
                let encoder = PngEncoder::new(writer);
                encoder.write_image(pixels, width, height, color_type.into())?;
            }
            EncodableFormats::Pnm => {
                let encoder = PnmEncoder::new(writer);
                encoder.write_image(pixels, width, height, color_type.into())?;
            }
            EncodableFormats::Qoi => {
                let encoder = QoiEncoder::new(writer);
                encoder.write_image(pixels, width, height, color_type.into())?;
            }
            EncodableFormats::Tga => {
                let encoder = TgaEncoder::new(writer);
                encoder.write_image(pixels, width, height, color_type.into())?;
            }
            EncodableFormats::Webp => {
                let encoder = WebPEncoder::new_lossless(writer);
                encoder.write_image(pixels, width, height, img.color().into())?;
            }
            EncodableFormats::Exr | EncodableFormats::Tiff => {
                let mut buffer = Cursor::new(Vec::new());
                
                match format {
                    EncodableFormats::Exr => {
                        let encoder = OpenExrEncoder::new(&mut buffer);
                        encoder.write_image(pixels, width, height, color_type.into())?;
                    }
                    EncodableFormats::Tiff => {
                        let encoder = TiffEncoder::new(&mut buffer);
                        encoder.write_image(pixels, width, height, color_type.into())?;
                    }
                    _ => unreachable!(),
                }
                writer.write_all(buffer.get_ref())?;
                writer.flush()?;
            }
        }
    }
    Ok(())
}