mod filters;
mod inout;
mod types;

use std::io::{Read, stdin};

use clap::{Parser, Subcommand};
use filters::{
    FilterResult,
    blob_detect::{BlobBackground, BlobColorMode},
    filter_from_command,
};
use image::ImageReader;

use inout::print_image;
use types::{AugeError, Color, DotColorSource, EncodableFormats, OutputKind, ResizeInput};

#[derive(Debug, Parser)]
#[command(version, about="Auge is a CLI image editing tool", long_about = None)]
struct Cli {
    #[arg(
        long,
        short,
        value_name = "FILE",
        help = "File to read from disk. If ommited STDIN is read."
    )]
    input: Option<String>,

    #[arg(long, short, value_enum, default_value = "png", help = "Output format")]
    format: EncodableFormats,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "A no-op image output.")]
    View,

    #[command(about = "Turn image into shades of gray")]
    Grayscale,

    #[command(about = "Apply gaussian blur")]
    GBlur {
        #[arg(long, short, help = "A measure of how much to blur by")]
        sigma: f32,
        #[arg(long, short, help = "Use fast, less accurate version")]
        fast: bool,
    },

    #[command(about = "Apply dot art filter")]
    Dotart {
        #[arg(long, short, value_enum, default_value = "raster")]
        output: OutputKind,
        #[arg(
            long,
            short,
            default_value = "16",
            help = "What area will one dot cover"
        )]
        scale: u32,
        #[arg(
            long,
            short = 'l',
            help = "Lower luma bound, e.g 0.1 = consider 10% darkest pixels as pitch black",
            default_value = "0.75"
        )]
        lower_percentile: f32,
        #[arg(
            long,
            short = 'u',
            help = "Upper luma bound, e.g. 0.1 = consider 10% brightest pixels as white",
            default_value = "0.1"
        )]
        upper_percentile: f32,
        #[arg(long, short = 'c', help = "Color of dots", default_value = "preserve")]
        dot_color: DotColorSource,
        #[arg(
            long,
            short = 'b',
            help = "Color for background",
            default_value = "#000000"
        )]
        bg_color: Color,
    },

    #[command(about = "Apply dynamic threshold filter")]
    Dynthres {
        #[arg(long, short = 'l', help = "Lower luma bound")]
        #[arg(
            long,
            short = 'l',
            help = "Lower luma bound, e.g 0.1 = consider 10% darkest pixels as black"
        )]
        lower_percentile: f32,
        #[arg(
            long,
            short = 'u',
            help = "Upper luma bound, e.g. 0.1 = consider 10% brightest pixels as white"
        )]
        upper_percentile: f32,

        #[arg(long, short = 'd', help = "Color to use for dark pixels")]
        dark_color: Option<Color>,
        #[arg(long, short = 'm', help = "Color to use for midtone pixels")]
        mid_color: Option<Color>,
        #[arg(long, short = 'b', help = "Color to use for bright pixels")]
        bright_color: Option<Color>,
    },

    #[command(about = "Resize image")]
    Resize {
        #[arg(long, short, help = "Use exact resizing")]
        exact: bool,
        #[arg(
            long,
            short,
            help = "Resolution target, in relative, or absolute format. Example values: 1980x1080, x1080 (equivalent to autox1080), 1920x (equivalent to 1920xauto), 120%"
        )]
        target: ResizeInput,
        #[arg(
            long,
            short,
            help = "Filter to use for resizing",
            default_value = "catmull-rom"
        )]
        filter: types::FilterType,
    },

    #[command(about = "Invert colors")]
    Invert,

    #[command(about = "Apply sepia tone filter")]
    Sepia,

    #[command(about = "Apply edge detection filter")]
    Edge,

    #[command(about = "Detect and colorize blobs of pixels")]
    BlobDetect {
        #[arg(
            long,
            short,
            default_value = "10",
            help = "Threshold (0-255). Pixels matching the target color within this threshold are considered part of the blob."
        )]
        threshold: u8,

        #[arg(long, short, help = "Target color to detect blobs of.")]
        color: Option<Color>,

        #[arg(
            long,
            short,
            value_enum,
            default_value = "rainbow",
            help = "Coloring strategy for blobs"
        )]
        mode: BlobColorMode,

        #[arg(
            long,
            short = 'b',
            value_enum,
            default_value = "black",
            help = "Background style"
        )]
        background: BlobBackground,
    },
}

fn main() -> Result<(), AugeError> {
    let cli = Cli::parse();

    let img = if let Some(path) = cli.input {
        ImageReader::open(&path)?.decode()?
    } else {
        let mut handle = stdin().lock();
        let mut buffer = Vec::new();
        handle.read_to_end(&mut buffer)?;
        image::load_from_memory(&buffer)?
    };

    let filter = filter_from_command(cli.command)?;
    match filter.apply(img)? {
        FilterResult::Image(img) => {
            print_image(&img, cli.format)?;
        }
        FilterResult::Text(text) => {
            println!("{}", text)
        }
    }

    Ok(())
}
