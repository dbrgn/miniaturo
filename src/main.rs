use std::path::{Path, PathBuf};
use std::io::Cursor;

use anyhow::{bail, Result};
use clap::Clap;
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use libopenraw_rs as libopenraw;

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

/// Convert this thumbnail to an image-rs `DynamicImage`.
fn to_image(thumbnail: &libopenraw::Thumbnail, _orientation: i32) -> Result<image::DynamicImage> {
    // Extract raw thumbnail data
    let data = thumbnail.get_data()?;

    // Convert depending on format
    let format = thumbnail.get_format();
    use libopenraw::DataType;
    Ok(match format {
        DataType::OR_DATA_TYPE_JPEG | DataType::OR_DATA_TYPE_PNG | DataType::OR_DATA_TYPE_TIFF => {
            let format = match format {
                DataType::OR_DATA_TYPE_JPEG => image::ImageFormat::Jpeg,
                DataType::OR_DATA_TYPE_PNG => image::ImageFormat::Png,
                DataType::OR_DATA_TYPE_TIFF => image::ImageFormat::Tiff,
                _ => unreachable!(),
            };

            let mut reader = image::io::Reader::new(Cursor::new(data));
            reader.set_format(format);
            reader.decode()?
        },
        DataType::OR_DATA_TYPE_PIXMAP_8RGB => {
            let (x, y) = thumbnail.get_dimensions();
            if let Some(img) = image::RgbImage::from_raw(x, y, data.to_vec()) {
                image::DynamicImage::ImageRgb8(img)
            } else {
                image::DynamicImage::ImageRgb8(image::RgbImage::new(x, y))
            }
        }
        _ => bail!("Unsupported thumbnail format: {:?}", format),
    })
}

fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let opts: Opts = Opts::parse();

    // Create a new rawfile
    let rawfile = libopenraw::RawFile::from_file(&opts.input_path, libopenraw::RawFileType::OR_RAWFILE_TYPE_UNKNOWN)?;

    // Get thumbnail
    let thumbnail = rawfile.get_thumbnail(opts.thumbnail_size)?;

    // Get orientation
    let orientation = rawfile.get_orientation();

    // Convert thumbnail to image-rs buffer
    let img = to_image(&thumbnail, orientation)?;

    // Write output file
    save_thumbnail(img, &opts.output_path, opts.thumbnail_size)?;

    // TODO exif rotate

    Ok(())
}
