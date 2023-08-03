//! GUI Image Processing
use std::fs::read_to_string;

use base64::{engine::general_purpose, Engine as _};
use cached::proc_macro::cached;
use resvg::usvg::TreeParsing;
use thiserror::Error;

#[derive(Debug, Error)]
enum SvgError {
    #[error("Invalid SVG Filepath")]
    InvalidFile(#[from] std::io::Error),
    #[error("Invalid Document")]
    InvalidTree(#[from] resvg::usvg::Error),
    #[error("Failed to Alloc PixBuf")]
    NoPixBuf,
    #[error("Failed to Convert SVG to PNG")]
    PngError(#[from] png::EncodingError),
}

fn svg_to_png(path: &str, pixels: u32) -> Result<String, SvgError> {
    // read and convert to resvg document tree
    let xml = read_to_string(path)?;
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(&xml, &opt)?;
    let rtree = resvg::Tree::from_usvg(&tree);
    // generate pixel-buffer and scale according to size preference
    let size = rtree.size.to_int_size();
    let scale = pixels / size.width();
    let width = size.width() * scale;
    let height = size.height() * scale;
    let fscale = scale as f32;
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(width, height).ok_or_else(|| SvgError::NoPixBuf)?;
    let form = resvg::tiny_skia::Transform::from_scale(fscale, fscale);
    // render as png to memory
    rtree.render(form, &mut pixmap.as_mut());
    let mut png = pixmap.encode_png()?;
    // base64 encode png
    let encoded = general_purpose::STANDARD.encode(&mut png);
    Ok(format!("data:image/png;base64, {encoded}"))
}

#[cached]
pub fn convert_svg(path: String) -> Option<String> {
    svg_to_png(&path, 64).ok()
}
