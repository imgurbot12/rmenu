use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use rmenu_plugin::Entry;
use serde::Deserialize;

use crate::WindowManager;

#[derive(Debug, Deserialize)]
pub struct HyprWindow {
    pub title: String,
    pub pid: u64,
    #[serde(alias = "focusHistoryID")]
    pub focus: u32,
}

pub fn get_windows() -> Result<Vec<HyprWindow>> {
    let out = Command::new("hyprctl")
        .args(["clients", "-j"])
        .stdout(Stdio::piped())
        .output()
        .context("Hyprctl failed to execute")?;
    if !out.status.success() {
        return Err(anyhow!("Invalid hyprctl Status: {:?}", out.status));
    }
    let mut windows: Vec<HyprWindow> =
        serde_json::from_slice(&out.stdout).context("Failed to parse hyprctl output")?;
    windows.sort_by_key(|w| w.focus);
    Ok(windows)
}

#[derive(Debug)]
pub struct HyprlandManager {}

impl WindowManager for HyprlandManager {
    /// Focus on Specified Window
    fn focus(&self, id: &str) -> Result<()> {
        let out = Command::new("hyprctl")
            .arg("dispatch")
            .arg("focuswindow")
            .arg(format!("pid:{id}"))
            .output()
            .context(format!("Failed Hyprctl to focus window: {id:?}"))?;
        if !out.status.success() {
            return Err(anyhow!("Hyprctl exited with error: {:?}", out.status));
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
                Entry::new(&w.title, &exec, None)
            })
            .collect();
        Ok(entries)
    }
}
