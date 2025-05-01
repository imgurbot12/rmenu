//! SVG to PNG Image Conversion
use std::fs::write;
use std::path::PathBuf;

use resvg::usvg::Options;
use thiserror::Error;

static XDG_PREFIX: &'static str = "rmenu";

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
#[inline]
pub fn make_temp() -> PathBuf {
    xdg::BaseDirectories::with_prefix(XDG_PREFIX)
        .expect("Failed to read xdg base dirs")
        .create_cache_directory("images")
        .expect("Failed to write xdg cache dirs")
}

/// Convert SVG to PNG Image
fn svg_to_png(path: &str, dest: &PathBuf, pixels: u32, opt: &Options) -> Result<(), SvgError> {
    // read and convert to resvg document tree
    let xml = std::fs::read(path)?;
    let tree = resvg::usvg::Tree::from_data(&xml, opt)?;
    // generate pixel-buffer and scale according to size preference
    let size = tree.size().to_int_size();
    let scale = pixels as f32 / size.width() as f32;
    let width = (size.width() as f32 * scale) as u32;
    let height = (size.height() as f32 * scale) as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| SvgError::NoPixBuf(width, height, pixels))?;
    let form = resvg::tiny_skia::Transform::from_scale(scale, scale);
    // render as png to memory
    resvg::render(&tree, form, &mut pixmap.as_mut());
    let png = pixmap.encode_png()?;
    // base64 encode png
    Ok(write(dest, png)?)
}

pub fn svg_options<'a>() -> Options<'a> {
    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    opt
}

pub fn svg_path(base: &PathBuf, path: &str) -> Option<PathBuf> {
    let (_, fname) = path.rsplit_once('/')?;
    let (name, _) = fname.rsplit_once(".")?;
    let name = format!("{name}.png");
    Some(base.join(name))
}

pub fn convert_svg(svg: &str, png: &PathBuf, opt: &Options) {
    log::debug!("generating png {png:?}");
    match svg_to_png(&svg, &png, 64, opt) {
        Err(err) => log::error!("failed svg->png: {err:?}"),
        _ => {}
    }
}
