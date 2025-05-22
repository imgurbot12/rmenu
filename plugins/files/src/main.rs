#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::io::{BufRead, BufReader};
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI16, Ordering};

use anyhow::Context;
use ignore::{DirEntry, Error, WalkState};
use regex::RegexBuilder;
use rmenu_plugin::{Entry, Message, Search};

static RESULT_LIMIT: i16 = 100;

// stolen from https://github.com/sharkdp/fd/blob/0d99badc1b2571e13359a09fbb1ff8e07c193cbe/src/cli.rs#L761-L772
fn default_num_threads() -> NonZeroUsize {
    let fallback = NonZeroUsize::MIN;
    let limit = NonZeroUsize::new(64).unwrap();
    std::thread::available_parallelism()
        .unwrap_or(fallback)
        .min(limit)
}

fn dirname(path: &Path) -> String {
    let dirname = path
        .parent()
        .map(|p| p.to_str().expect("invalid path"))
        .unwrap_or(".");
    match dirname.is_empty() {
        false => dirname,
        true => ".",
    }
    .to_owned()
}

fn print_stop() -> anyhow::Result<()> {
    let message = Message::Stop;
    let str = serde_json::to_string(&message).context("failed to serialize stop")?;
    println!("{str}");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let home = shellexpand::tilde("~").to_string();
    std::env::set_current_dir(home).expect("failed to change directory");

    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin);

    let mut signals = vec![];
    let mut threads = vec![];
    let num_threads = default_num_threads().get();
    loop {
        // read search message from stdin
        let mut buf = String::new();
        match reader.read_line(&mut buf) {
            Ok(_) => {}
            Err(err) => {
                log::error!("failed to read stdin: {err:?}");
                break;
            }
        }
        buf = buf.trim().to_owned();
        if buf.is_empty() {
            log::debug!("message empty. breaking loop.");
            break;
        }
        // trigger all existing signals
        while !signals.is_empty() {
            let signal: Arc<AtomicBool> = signals.pop().expect("signal missing");
            signal.store(true, Ordering::SeqCst);
        }
        // build thread signal
        let condvar = Arc::new(AtomicBool::new(false));
        signals.push(Arc::clone(&condvar));
        // split dirname/basename from search
        let search: Search = serde_json::from_str(&buf).context("invalid search message")?;
        let query = shellexpand::tilde(&search.search).to_string();
        let path = Path::new(&query);
        let (dirn, pattern) = match path.is_dir() {
            true => (
                path.to_str().expect("invalid path").to_owned(),
                String::new(),
            ),
            false => {
                let search = path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .expect("invalid basename string")
                    .to_owned();
                let dirn = dirname(&path);
                match Path::new(&dirn).is_dir() {
                    true => (dirn, search),
                    false => (".".to_string(), query),
                }
            }
        };
        log::info!("searching directory={dirn:?} pattern={:?}", pattern);
        // determine depth
        let depth = match pattern.is_empty() {
            true => Some(1),
            false => None,
        };
        // recurse files matching search
        let walk = ignore::WalkBuilder::new(dirn)
            .hidden(true)
            .ignore(true)
            .git_ignore(false)
            .max_depth(depth)
            .threads(num_threads)
            .build_parallel();
        // build regex pattern
        let filter = match RegexBuilder::new(&pattern)
            .case_insensitive(search.ignore_case)
            .build()
        {
            Ok(rgx) => rgx,
            Err(err) => {
                log::warn!("pattern error: {err:?}");
                print_stop()?;
                continue;
            }
        };
        // iterate results
        let thread = std::thread::spawn(move || {
            let counter = Arc::new(AtomicI16::new(0));
            walk.run(|| {
                let c = Arc::clone(&counter);
                let rx = Arc::clone(&condvar);
                let pattern = filter.clone();
                Box::new(move |entry: Result<DirEntry, Error>| {
                    if rx.load(Ordering::Relaxed) {
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
                    if let Ok(val) =
                        c.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| Some(v + 1))
                    {
                        if val >= RESULT_LIMIT {
                            return WalkState::Quit;
                        }
                    }
                    let path = entry.path();
                    let full = path.canonicalize().unwrap_or(path.to_path_buf());
                    let action = rmenu_plugin::extra::open_command(full);
                    let comment = format!("{path:?}");
                    let entry = Entry::new(&comment, &action, None);
                    let json = serde_json::to_string(&entry).expect("failed to serialize message");
                    println!("{json}");
                    WalkState::Continue
                })
            });
            // report finished and reset boolean
            print_stop().expect("failed to send stop message");
        });
        threads.push(thread);
    }
    for thread in threads {
        let _ = thread.join();
    }
    Ok(())
}
