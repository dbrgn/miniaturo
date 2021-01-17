use std::{
    convert::TryInto,
    ffi::{CString, OsStr},
    io::Cursor,
    path::{Path, PathBuf},
};

// Note: Windows is not yet supported.
// See https://internals.rust-lang.org/t/pathbuf-to-cstring/12560 for details.
use std::os::unix::ffi::OsStrExt;

use anyhow::{bail, Result};
use clap::Clap;
use image::{imageops::FilterType, io::Reader as ImageReader, DynamicImage, ImageFormat};

mod libopenraw;

use libopenraw::ffi;
use libopenraw::ffi::ORThumbnailRef;

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

/// Convert a libopenraw thumbnail data type to an image-rs `ImageFormat`.
fn data_type_to_image_format(data_type: ffi::or_data_type::Type) -> Result<ImageFormat> {
    Ok(match data_type {
        ffi::or_data_type::OR_DATA_TYPE_JPEG => ImageFormat::Jpeg,
        ffi::or_data_type::OR_DATA_TYPE_TIFF => ImageFormat::Tiff,
        ffi::or_data_type::OR_DATA_TYPE_PNG => ImageFormat::Png,
        other => bail!("Unsupported thumbnail format: {}", other),
    })
}

fn thumbnail_to_image(thumbnail: ORThumbnailRef, orientation: i32) -> Result<DynamicImage> {
    // Extract raw thumbnail data
    let format = unsafe { ffi::or_thumbnail_format(thumbnail) };
    let buf_size = unsafe { ffi::or_thumbnail_data_size(thumbnail) };
    let input_buf: &[u8] = unsafe {
        std::slice::from_raw_parts(
            ffi::or_thumbnail_data(thumbnail) as *const u8,
            buf_size.try_into().unwrap(),
        )
    };

    Ok(match format {
        ffi::or_data_type::OR_DATA_TYPE_JPEG
        | ffi::or_data_type::OR_DATA_TYPE_PNG
        | ffi::or_data_type::OR_DATA_TYPE_TIFF => {
            let mut reader = ImageReader::new(Cursor::new(input_buf));
            reader.set_format(data_type_to_image_format(format)?);
            reader.decode()?
        }
        _ => bail!("Unsupported thumbnail format: {}", format),
    })
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
    let input_path_os_str: &OsStr = &opts.input_path.as_ref();
    let input_path = CString::new(input_path_os_str.as_bytes()).unwrap();
    let rawfile = unsafe {
        ffi::or_rawfile_new(
            input_path.as_ptr(),
            ffi::or_rawfile_type::OR_RAWFILE_TYPE_UNKNOWN,
        )
    };
    if rawfile.is_null() {
        bail!("Could not create rawfile");
    }

    // Get thumbnail
    let thumbnail = unsafe { ffi::or_thumbnail_new() };
    let err = unsafe { ffi::or_rawfile_get_thumbnail(rawfile, opts.thumbnail_size, thumbnail) };
    if err != ffi::or_error::OR_ERROR_NONE {
        // TODO: Parse error code into enum
        bail!("Extracting thumbnail data failed with error code {}", err);
    }

    // Get orientation
    let orientation = unsafe { ffi::or_rawfile_get_orientation(rawfile) };

    // Convert thumbnail to image-rs buffer
    let img = thumbnail_to_image(thumbnail, orientation)?;

    // Release thumbnail memory
    let err = unsafe { ffi::or_thumbnail_release(thumbnail) };
    if err != ffi::or_error::OR_ERROR_NONE {
        eprintln!("Warning: Could not free thumbnail memory: {}", err);
    }

    // Release rawfile
    let err = unsafe { ffi::or_rawfile_release(rawfile) };
    if err != ffi::or_error::OR_ERROR_NONE {
        eprintln!("Warning: Could not free rawfile memory: {}", err);
    }

    // Write output file
    save_thumbnail(img, &opts.output_path, opts.thumbnail_size)?;

    // TODO exif rotate

    Ok(())
}
