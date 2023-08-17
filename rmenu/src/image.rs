//! GUI Image Processing
use std::fs::{create_dir_all, read_to_string, write};
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use cached::proc_macro::cached;
use once_cell::sync::Lazy;
use resvg::usvg::TreeParsing;
use thiserror::Error;

static TEMP_EXISTS: Lazy<Mutex<Vec<bool>>> = Lazy::new(|| Mutex::new(vec![]));
static TEMP_DIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("/tmp/rmenu"));

#[derive(Debug, Error)]
enum SvgError {
    #[error("Invalid SVG Filepath")]
    InvalidFile(#[from] std::io::Error),
    #[error("Invalid Document")]
    InvalidTree(#[from] resvg::usvg::Error),
    #[error("Failed to Alloc PixBuf")]
    NoPixBuf(u32, u32, u32),
    #[error("Failed to Convert SVG to PNG")]
    PngError(#[from] png::EncodingError),
}

/// Make Temporary Directory for Generated PNGs
fn make_temp() -> Result<(), io::Error> {
    let mut temp = TEMP_EXISTS.lock().expect("Failed to Access Global Mutex");
    if temp.len() == 0 {
        create_dir_all(TEMP_DIR.to_owned())?;
        temp.push(true);
    }
    Ok(())
}

/// Convert SVG to PNG Image
fn svg_to_png(path: &str, dest: &PathBuf, pixels: u32) -> Result<(), SvgError> {
    // read and convert to resvg document tree
    let xml = read_to_string(path)?;
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(&xml, &opt)?;
    let rtree = resvg::Tree::from_usvg(&tree);
    // generate pixel-buffer and scale according to size preference
    let size = rtree.size.to_int_size();
    let scale = pixels as f32 / size.width() as f32;
    let width = (size.width() as f32 * scale) as u32;
    let height = (size.height() as f32 * scale) as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| SvgError::NoPixBuf(width, height, pixels))?;
    let form = resvg::tiny_skia::Transform::from_scale(scale, scale);
    // render as png to memory
    rtree.render(form, &mut pixmap.as_mut());
    let png = pixmap.encode_png()?;
    // base64 encode png
    Ok(write(dest, png)?)
}

#[cached]
pub fn convert_svg(path: String) -> Option<String> {
    // ensure temporary directory exists
    let _ = make_temp();
    // convert path to new temporary png filepath
    let (_, fname) = path.rsplit_once('/')?;
    let (name, _) = fname.rsplit_once(".")?;
    let name = format!("{name}.png");
    let new_path = TEMP_DIR.join(name);
    // generate png if it doesnt already exist
    if !new_path.exists() {
        log::debug!("generating png {new_path:?}");
        match svg_to_png(&path, &new_path, 64) {
            Err(err) => log::error!("failed svg->png: {err:?}"),
            _ => {}
        }
    }
    Some(new_path.to_str()?.to_string())
}

#[cached]
pub fn image_exists(path: String) -> bool {
    PathBuf::from(path).exists()
}
