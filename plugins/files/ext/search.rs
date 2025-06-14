//! FileSearch Manager

use std::num::NonZeroUsize;
use std::path::PathBuf;

use anyhow::{Context, Result};
use ignore::{DirEntry, Error, WalkState};
use regex::RegexBuilder;
use rmenu_plugin::{Entry, Search};

use crate::{
    atomic::{Counter, Signal},
    print_stop,
};

//TODO: build FileManager that maintains state and tracks the results
// of the last completed lookup

// stolen from https://github.com/sharkdp/fd/blob/0d99badc1b2571e13359a09fbb1ff8e07c193cbe/src/cli.rs#L761-L772
fn default_num_threads() -> NonZeroUsize {
    let fallback = NonZeroUsize::MIN;
    let limit = NonZeroUsize::new(64).unwrap();
    std::thread::available_parallelism()
        .unwrap_or(fallback)
        .min(limit)
}

pub struct FileSearch {
    dir: PathBuf,
    pattern: String,
    search: Search,
    signal: Signal,
    thread_limit: usize,
    result_limit: usize,
}

impl FileSearch {
    pub fn search(self) -> Result<std::thread::JoinHandle<()>> {
        let filter = RegexBuilder::new(&self.pattern)
            .case_insensitive(self.search.ignore_case)
            .build()
            .context("regex pattern error")?;
        let depth = match self.pattern.is_empty() {
            true => Some(1),
            false => None,
        };
        let walk = ignore::WalkBuilder::new(self.dir)
            .hidden(true)
            .ignore(true)
            .git_ignore(false)
            .max_depth(depth)
            .threads(self.thread_limit)
            .build_parallel();
        let signal = self.signal.clone();
        Ok(std::thread::spawn(move || {
            let counter = Counter::new();
            walk.run(|| {
                let counter = counter.clone();
                let signal = signal.clone();
                let pattern = filter.clone();
                Box::new(move |entry: Result<DirEntry, Error>| {
                    if signal.is_tripped() {
                        return WalkState::Quit;
                    }
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(err) => {
                            log::error!("failed to read: {err:?}");
                            return WalkState::Continue;
                        }
                    };
                    let path = entry.path();
                    let path_str = path.to_str().expect("invalid path string");
                    if !pattern.is_match(path_str) || path_str == "." {
                        return WalkState::Continue;
                    }
                    if counter.tick() >= self.result_limit {
                        return WalkState::Quit;
                    }
                    let path = entry.path();
                    let full = path.canonicalize().unwrap_or(path.to_path_buf());
                    let action = format!("xdg-open {full:?}");
                    let comment = format!("{path:?}");
                    let entry = Entry::new(&comment, &action, None);
                    let json = serde_json::to_string(&entry).expect("failed to serialize message");
                    println!("{json}");
                    WalkState::Continue
                })
            });
            // report finished and reset boolean
            print_stop().expect("failed to send stop message");
        }))
    }
}
