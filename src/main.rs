use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Clap;
use image::{imageops::FilterType, DynamicImage, ImageFormat};

mod libopenraw;

#[derive(Clap)]
#[clap(about, version)]
struct Opts {
    /// Input file
    #[clap(short = 'i')]
    input_path: PathBuf,

    /// Output file (PNG)
    #[clap(short = 'o')]
    output_path: PathBuf,

    /// Thumbnail size
    #[clap(short = 's', default_value = "128")]
    thumbnail_size: u32,
}

/// Save the image data `img` to `output_path`.
///
/// The sampling filter used is Catmull-Rom. It offers a good balance between
/// performance and quality. See [image-rs
/// docs](https://docs.rs/image/*/image/imageops/enum.FilterType.html) for more
/// details.
///
/// For compatibility with raw-thumbnailer, the output format is always PNG.
fn save_thumbnail(img: DynamicImage, output_path: &Path, thumbnail_size: u32) -> Result<()> {
    img.resize(thumbnail_size, thumbnail_size, FilterType::CatmullRom)
        .save_with_format(output_path, ImageFormat::Png)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let opts: Opts = Opts::parse();

    // Create a new rawfile
    let rawfile = libopenraw::RawFile::new(&opts.input_path)?;

    // Get thumbnail
    let thumbnail = rawfile.get_thumbnail(opts.thumbnail_size)?;

    // Get orientation
    let orientation = rawfile.get_orientation();

    // Convert thumbnail to image-rs buffer
    let img = thumbnail.to_image(orientation)?;

    // Write output file
    save_thumbnail(img, &opts.output_path, opts.thumbnail_size)?;

    // TODO exif rotate

    Ok(())
}
