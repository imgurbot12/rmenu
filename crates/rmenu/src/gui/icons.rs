/*
 *  GUI Icon Cache/Loading utilities
 */
use std::sync::Arc;
use std::thread;

use dashmap::{mapref::one::Ref, DashMap};
use egui_extras::RetainedImage;
use log::debug;

use rmenu_plugin::{Entry, Icon};

/* Types */

type Cache = DashMap<String, RetainedImage>;
type IconRef<'a> = Ref<'a, String, RetainedImage>;

/* Functions */

// load result entry icons into cache in background w/ given chunk-size per thread
pub fn load_images(cache: &mut IconCache, chunk_size: usize, results: &Vec<Entry>) {
    // retrieve icons from results
    let icons: Vec<Icon> = results
        .iter()
        .filter_map(|r| r.icon.clone().into())
        .collect();
    for chunk in icons.chunks(chunk_size).into_iter() {
        cache.save_background(Vec::from(chunk));
    }
}

/* Cache */

// spawn multiple threads to load image objects into cache from search results

pub struct IconCache {
    c: Arc<Cache>,
}

impl IconCache {
    pub fn new() -> Self {
        Self {
            c: Arc::new(Cache::new()),
        }
    }

    // save icon to cache if not already saved
    pub fn save(&mut self, icon: &Icon) -> Result<(), String> {
        let name = icon.name.as_str();
        let path = icon.path.as_str();
        if !self.c.contains_key(name) {
            self.c.insert(
                name.to_owned(),
                if path.ends_with(".svg") {
                    RetainedImage::from_svg_bytes(path, &icon.data)?
                } else {
                    RetainedImage::from_image_bytes(path, &icon.data)?
                },
            );
        }
        Ok(())
    }

    // load an image from the given icon-cache
    #[inline]
    pub fn load(&mut self, icon: &Icon) -> Result<IconRef<'_>, String> {
        self.save(icon)?;
        Ok(self
            .c
            .get(icon.name.as_str())
            .expect("failed to load saved image"))
    }

    // save list of icon-entries in the background
    pub fn save_background(&mut self, icons: Vec<Icon>) {
        let mut cache = self.clone();
        thread::spawn(move || {
            for icon in icons.iter() {
                if let Err(err) = cache.save(&icon) {
                    debug!("icon {} failed to load: {}", icon.name.as_str(), err);
                };
            }
            debug!("background task loaded {} icons", icons.len());
        });
    }
}

impl Clone for IconCache {
    fn clone(&self) -> Self {
        Self {
            c: Arc::clone(&self.c),
        }
    }
}
