//! Sway WindowMangager Window Selector
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use rmenu_plugin::Entry;
use serde::Deserialize;
use serde_json::Value;

use crate::WindowManager;

static SWAY_TYPE_KEY: &'static str = "type";
static SWAY_NODES_KEY: &'static str = "nodes";
static SWAY_WINDOW_TYPE: &'static str = "con";
static SWAY_WINDOW_NAME: &'static str = "name";

#[derive(Debug, Deserialize)]
pub struct SwayWindow {
    pub name: String,
    pub pid: u64,
    pub focused: bool,
}

#[derive(Debug)]
pub struct SwayManager {}

pub fn get_windows() -> Result<Vec<SwayWindow>> {
    // retrieve output of swaymsg tree
    let out = Command::new("swaymsg")
        .args(["-t", "get_tree"])
        .stdout(Stdio::piped())
        .output()
        .context("SwayMsg Failed to Execute")?;
    if !out.status.success() {
        return Err(anyhow!("Invalid SwayMsg Status: {:?}", out.status));
    }
    // read output as string
    let result: Value =
        serde_json::from_slice(&out.stdout).context("Failed to Parse SwayMsg Output")?;
    // recursively parse object for window definitions
    let mut nodes = vec![result];
    let mut windows = vec![];
    while let Some(item) = nodes.pop() {
        if !item.is_object() {
            return Err(anyhow!("Unexpected Node Value: {:?}", item));
        }
        // pass additional nodes if not a valid window object
        let Some(ntype) = item.get(SWAY_TYPE_KEY) else { continue };
        let is_nulled = item
            .get(SWAY_WINDOW_NAME)
            .map(|v| v.is_null())
            .unwrap_or(false);
        if ntype != SWAY_WINDOW_TYPE || is_nulled {
            let Some(snodes) = item.get(SWAY_NODES_KEY) else { continue };
            match snodes {
                Value::Array(array) => nodes.extend(array.clone().into_iter()),
                _ => return Err(anyhow!("Unexpected NodeList Value: {:?}", snodes)),
            }
            continue;
        }
        let window: SwayWindow =
            serde_json::from_value(item.clone()).context("Failed to Parse Window Object")?;
        windows.push(window);
    }
    windows.sort_by_key(|w| w.focused);
    Ok(windows)
}

impl WindowManager for SwayManager {
    /// Focus on Specified Window
    fn focus(&self, id: &str) -> Result<()> {
        let out = Command::new("swaymsg")
            .arg(format!("[pid={}] focus", id))
            .output()
            .context("Failed SwayMsg To Focus Window: {id:?}")?;
        if !out.status.success() {
            return Err(anyhow!("SwayMsg Exited with Error: {:?}", out.status));
        }
        Ok(())
    }
    /// Generate RMenu Entries
    fn entries(&self) -> Result<Vec<Entry>> {
        let exe = std::env::current_exe()?.to_str().unwrap().to_string();
        let windows = get_windows()?;
        let entries = windows
            .into_iter()
            .map(|w| {
                let exec = format!("{exe} focus {:?}", w.pid);
                Entry::new(&w.name, &exec, None)
            })
            .collect();
        Ok(entries)
    }
}
