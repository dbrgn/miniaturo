use std::io::Cursor;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use clap::{arg, command, Parser};
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use libopenraw_rs as libopenraw;

#[derive(Parser, Debug)]
#[command(
    about,
    author = clap::crate_authors!("\n"),
    version,
    help_template = "{before-help}{name} {version}
{about-with-newline}{author-with-newline}
{usage-heading} {usage}

{all-args}{after-help}",
    after_help = "GitHub: https://github.com/dbrgn/miniaturo"
)]
struct Opts {
    /// Input file
    #[arg(short = 'i')]
    input_path: PathBuf,

    /// Output file (PNG)
    #[arg(short = 'o')]
    output_path: PathBuf,

    /// Thumbnail size
    #[arg(short = 's', default_value = "128")]
    thumbnail_size: u32,
}

/// Describe how to rotate the image pixels (clockwise) in order to get a
/// straight picture.
#[derive(Debug)]
enum Rotate {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[derive(Debug)]
struct ExifOrientation {
    rotate: Rotate,
    mirrored: bool,
}

impl ExifOrientation {
    /// Create an [`ExifOrientation`] from an orientation integer as returned
    /// by libopenraw.
    ///
    /// See <https://jdhao.github.io/2019/07/31/image_rotation_exif_info/> for
    /// more details on EXIF rotation values.
    fn from_exif(orientation: i32) -> Result<Self> {
        let rotate = match orientation {
            0 | 1 | 2 => Rotate::Deg0,
            3 | 4 => Rotate::Deg180,
            5 | 6 => Rotate::Deg90,
            7 | 8 => Rotate::Deg270,
            _ => bail!("Invalid exif orientation: {}", orientation),
        };
        let mirrored = match orientation {
            0 | 1 | 3 | 6 | 8 => false,
            2 | 4 | 5 | 7 => true,
            _ => bail!("Invalid exif orientation: {}", orientation),
        };
        Ok(Self { rotate, mirrored })
    }
}

/// Save the image data `img` to `output_path` with proper size and
/// orientation.
///
/// The sampling filter used for scaling is Catmull-Rom. It offers a good
/// balance between performance and quality. See [image-rs
/// docs](https://docs.rs/image/*/image/imageops/enum.FilterType.html) for more
/// details.
///
/// For compatibility with raw-thumbnailer, the output format is always PNG.
fn save_thumbnail(
    img: DynamicImage,
    output_path: &Path,
    thumbnail_size: u32,
    orientation: ExifOrientation,
) -> Result<()> {
    // Resize thumbnail
    let resized = img.resize(thumbnail_size, thumbnail_size, FilterType::CatmullRom);

    // Mirror and rotate
    let flipped = match orientation.mirrored {
        true => resized.fliph(),
        false => resized,
    };
    let rotated = match orientation.rotate {
        Rotate::Deg0 => flipped,
        Rotate::Deg90 => flipped.rotate90(),
        Rotate::Deg180 => flipped.rotate180(),
        Rotate::Deg270 => flipped.rotate270(),
    };

    rotated.save_with_format(output_path, ImageFormat::Png)?;
    Ok(())
}

/// Convert this thumbnail to an image-rs `DynamicImage`.
fn to_image(thumbnail: &libopenraw::Thumbnail) -> Result<image::DynamicImage> {
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
        }
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
    let rawfile = libopenraw::RawFile::from_file(
        &opts.input_path,
        libopenraw::RawFileType::OR_RAWFILE_TYPE_UNKNOWN,
    )?;

    // Get thumbnail
    let thumbnail = rawfile.get_thumbnail(opts.thumbnail_size)?;

    // Get orientation
    let orientation = ExifOrientation::from_exif(rawfile.get_orientation())?;

    // Convert thumbnail to image-rs buffer
    let img = to_image(&thumbnail)?;

    // Write output file
    save_thumbnail(img, &opts.output_path, opts.thumbnail_size, orientation)?;

    // TODO exif rotate

    Ok(())
}
