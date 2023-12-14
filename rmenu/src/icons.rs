//! GUI Image Processing
use std::collections::HashMap;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;

use base64::{engine::general_purpose, Engine as _};
use rayon::prelude::*;
use resvg::usvg::TreeParsing;
use rmenu_plugin::Entry;
use thiserror::Error;

static TEMP_DIR: &'static str = "/tmp/rmenu";

#[derive(Debug, Error)]
pub enum SvgError {
    #[error("Invalid SVG Filepath")]
    InvalidFile(#[from] std::io::Error),
    #[error("Invalid Document")]
    InvalidTree(#[from] resvg::usvg::Error),
    #[error("Failed to Alloc PixBuf")]
    NoPixBuf(u32, u32, u32),
    #[error("Failed to Convert SVG to PNG")]
    PngError(#[from] png::EncodingError),
}

#[inline]
fn encode(data: Vec<u8>) -> String {
    general_purpose::STANDARD_NO_PAD.encode(data)
}

/// Convert SVG to PNG Image
fn svg_to_png(path: &str, dest: &PathBuf, pixels: u32) -> Result<Vec<u8>, SvgError> {
    // read and convert to resvg document tree
    let xml = std::fs::read(path)?;
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&xml, &opt)?;
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
    write(dest, png.clone())?;
    Ok(png)
}

#[derive(Debug)]
pub struct IconCache {
    path: PathBuf,
    rendered: HashMap<String, Option<String>>,
}

impl IconCache {
    pub fn new() -> Result<Self, SvgError> {
        let path = PathBuf::from(TEMP_DIR);
        create_dir_all(&path)?;
        Ok(Self {
            path,
            rendered: HashMap::new(),
        })
    }

    fn convert_svg(&self, path: &str) -> Option<Vec<u8>> {
        // convert path to new temporary png filepath
        let (_, fname) = path.rsplit_once('/')?;
        let (name, _) = fname.rsplit_once(".")?;
        let name = format!("{name}.png");
        let new_path = self.path.join(name);
        // generate png if it doesnt already exist
        if !new_path.exists() {
            log::debug!("generating png {new_path:?}");
            match svg_to_png(&path, &new_path, 64) {
                Err(err) => log::error!("failed svg->png: {err:?}"),
                Ok(data) => return Some(data),
            }
        }
        std::fs::read(new_path).ok()
    }

    /// Prepare and PreGenerate Icon Images
    pub fn prepare(&mut self, entries: &[&Entry]) {
        let icons: Vec<(String, Option<String>)> = entries
            .into_par_iter()
            .filter_map(|e| e.icon.as_ref())
            .filter(|i| !self.rendered.contains_key(i.to_owned()))
            .filter_map(|path| {
                if path.ends_with(".png") {
                    let result = std::fs::read(path).ok().map(encode);
                    return Some((path.clone(), result));
                }
                if path.ends_with(".svg") {
                    let result = self.convert_svg(&path).map(encode);
                    return Some((path.clone(), result));
                }
                None
            })
            .collect();
        self.rendered.extend(icons);
    }

    // locate cached icon from specified path (if given)
    pub fn locate(&self, icon: &Option<String>) -> &Option<String> {
        let Some(path) = icon else { return &None };
        if self.rendered.contains_key(path) {
            return self.rendered.get(path).unwrap();
        }
        &None
    }
}
