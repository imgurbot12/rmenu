/// RMenu Plugin Result Entry Server
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use rmenu_plugin::{Entry, Message, Search};
use thiserror::Error;

use super::config::{Config, Format, PluginConfig};
use super::search::new_searchfn;

#[derive(Error, Debug)]
pub enum RMenuError {
    #[error("Invalid Config")]
    InvalidConfig(#[from] serde_yaml::Error),
    #[error("File Error")]
    FileError(#[from] std::io::Error),
    #[error("No Such Plugin")]
    NoSuchPlugin(String),
    #[error("Invalid Plugin Specified")]
    InvalidPlugin(String),
    #[error("Invalid Keybind Definition")]
    InvalidKeybind(String),
    #[error("Command Runtime Exception")]
    CommandError(Option<ExitStatus>),
    #[error("Invalid JSON Entry Object")]
    InvalidJson(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, RMenuError>;

#[derive(Debug)]
enum Cmd {
    Skipped,
    NotStarted,
    Started(Child),
}

impl Cmd {
    pub fn child(&mut self) -> &mut Child {
        match self {
            Self::Started(child) => child,
            _ => panic!("child not started"),
        }
    }
    pub fn kill(&mut self) -> std::result::Result<(), std::io::Error> {
        if let Self::Started(child) = self {
            child.kill()?
        }
        Ok(())
    }
}

fn new_search(query: &str, config: &Config) -> Search {
    Search {
        search: query.to_owned(),
        is_regex: config.search.use_regex,
        ignore_case: config.search.ignore_case,
    }
}

fn read_entries<T: Read>(
    format: &Format,
    config: &mut Config,
    reader: BufReader<T>,
) -> Result<Vec<Entry>> {
    let mut entries = vec![];
    for line in reader
        .lines()
        .filter_map(|l| l.ok())
        .map(|l| l.trim().to_owned())
        .filter(|l| !l.is_empty())
    {
        match format {
            Format::DMenu => entries.push(Entry::echo(&line, None)),
            Format::Json => {
                let msg: Message = serde_json::from_str(&line)?;
                match msg {
                    Message::Stop => break,
                    Message::Options(options) => config
                        .update(&options)
                        .map_err(|s| RMenuError::InvalidKeybind(s))?,
                    Message::Entry(mut entry) => {
                        //NOTE: windows paths and paths with predefined
                        // extensions like file:// or c:// are broken.
                        // (https://github.com/DioxusLabs/dioxus/issues/1814)
                        if let Some(icon) = entry.icon.as_ref() {
                            if icon.starts_with("C:") || icon.starts_with("D:") {
                                let path: Vec<String> =
                                    icon[2..].split("\\").map(|c| c.to_string()).collect();
                                let icon =
                                    format!("http://dioxus.{}{}", &icon[..2], path.join("/"));
                                entry.icon = Some(icon)
                            }
                        }
                        entries.push(entry)
                    }
                }
            }
        }
    }
    Ok(entries)
}

#[derive(Default)]
pub struct ServerBuilder {
    order: Vec<String>,
    sources: HashMap<String, Source>,
}

impl ServerBuilder {
    pub fn add_input(mut self, format: Format, input: &str, config: &mut Config) -> Result<Self> {
        if input == "-" {
            let stdin = std::io::stdin();
            let name = "stdin".to_owned();

            let input = Entries::new_file(format, config, stdin)?;
            self.order.push(name.to_owned());
            self.sources.insert(name, Source::Results(input));
            return Ok(self);
        }

        let input = shellexpand::tilde(input).to_string();
        let path = PathBuf::from(&input);
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or("unknown".to_string());

        let input = Input::new(format, path)?;
        self.order.push(name.to_owned());
        self.sources.insert(name, Source::Input(input));
        Ok(self)
    }

    pub fn add_plugin(mut self, name: String, config: &mut Config) -> Result<Self> {
        let cfg = config
            .plugins
            .get(&name)
            .cloned()
            .ok_or_else(|| RMenuError::NoSuchPlugin(name.to_owned()))?;
        if let Some(options) = cfg.options.as_ref() {
            config
                .update(options)
                .map_err(|e| RMenuError::InvalidKeybind(e))?;
        }
        let plugin = Plugin::new(name.to_owned(), &cfg)?;
        self.order.push(name.to_owned());
        self.sources.insert(name.to_owned(), Source::Plugin(plugin));
        Ok(self)
    }

    pub fn add_plugins(mut self, names: Vec<String>, config: &mut Config) -> Result<Self> {
        for name in names {
            self = self.add_plugin(name, config)?;
        }
        Ok(self)
    }

    pub fn build(self, mut show: Vec<String>) -> Result<Server> {
        for name in show.iter() {
            if !self.sources.contains_key(name) {
                return Err(RMenuError::InvalidPlugin(name.to_owned()));
            }
        }
        if show.is_empty() {
            let mode = self.order.get(0).expect("no active plugins").clone();
            log::warn!("no mode specified. defaulting to {mode:?}");
            show.push(mode);
        }
        return Ok(Server {
            order: self.order,
            sources: self.sources,
            active: show,
        });
    }
}

pub struct Server {
    order: Vec<String>,
    active: Vec<String>,
    sources: HashMap<String, Source>,
}

impl Server {
    pub fn search(&mut self, config: &mut Config, query: &str) -> Result<Vec<Entry>> {
        let mut results = vec![];
        for name in self.active.iter() {
            let plugin = self.sources.get_mut(name).expect("plugin missing");
            let entries = plugin.search(config, query)?;
            results.extend(entries);
        }
        Ok(results)
    }

    pub fn placeholder(&self, config: &Config) -> String {
        let mode = self.active.last().expect("no active plugins");
        let plugin = config
            .plugins
            .get(mode)
            .map(|c| c.placeholder.clone())
            .unwrap_or_default();
        config
            .search
            .placeholder
            .clone()
            .or(plugin)
            .unwrap_or_default()
    }

    fn mode_index(&self) -> usize {
        let mode = self.active.last().expect("no active plugins");
        self.order
            .iter()
            .position(|m| m == mode)
            .expect("invalid mode")
    }

    pub fn next_plugin(&mut self) {
        let index = self.mode_index();
        let mode = match index == self.order.len() - 1 {
            true => self.order.first().expect("no plugins avaialble"),
            false => self.order.get(index + 1).expect("cannot find next plugin"),
        };
        log::info!("switching to next mode: {mode:?}");
        self.active = vec![mode.to_owned()];
    }

    pub fn prev_plugin(&mut self) {
        let index = self.mode_index();
        let mode = match index == 0 {
            true => self.order.last().expect("no plugins avaialble"),
            false => self.order.get(index - 1).expect("cannot find prev plugin"),
        };
        log::info!("switching to prev mode: {mode:?}");
        self.active = vec![mode.to_owned()];
    }

    pub fn cleanup(&mut self) {
        let mut threads = vec![];
        for source in self.sources.values_mut() {
            if let Source::Plugin(plugin) = source {
                if let Some(thread) = plugin.cache_thread.take() {
                    threads.push(thread);
                }
                if let Err(err) = plugin.command.kill() {
                    log::warn!("failed to kill {:?} {:?}", plugin.name, err);
                }
            }
        }
        log::debug!("cleaning up {} threads", threads.len());
        while !threads.is_empty() {
            let thread = threads.pop().unwrap();
            let _ = thread.join();
        }
    }
}

enum Source {
    Input(Input),
    Plugin(Plugin),
    Results(Entries),
}

impl Source {
    pub fn search(&mut self, config: &mut Config, query: &str) -> Result<Vec<Entry>> {
        match self {
            Self::Input(input) => input.search(config, query),
            Self::Plugin(plugin) => plugin.search(config, query),
            Self::Results(results) => results.search(config, query),
        }
    }
}

struct Entries(Vec<Entry>);

impl Entries {
    pub fn new_file(format: Format, config: &mut Config, mut reader: impl Read) -> Result<Self> {
        let reader = BufReader::new(&mut reader);
        let entries = read_entries(&format, config, reader)?;
        Ok(Self(entries))
    }

    pub fn search(&mut self, config: &mut Config, query: &str) -> Result<Vec<Entry>> {
        let search = new_search(query, &config);
        let filter = new_searchfn(&search);
        Ok(self.0.clone().into_iter().filter(|e| filter(e)).collect())
    }
}

struct Input {
    input: PathBuf,
    format: Format,
    results: Option<Vec<Entry>>,
}

impl Input {
    pub fn new(format: Format, input: PathBuf) -> Result<Self> {
        Ok(Self {
            input,
            format,
            results: None,
        })
    }

    pub fn search(&mut self, config: &mut Config, query: &str) -> Result<Vec<Entry>> {
        let search = new_search(query, &config);
        let entries = match self.results.as_ref() {
            Some(results) => results.clone(),
            None => {
                log::info!("reading from: {:?}", self.input);
                let path = File::open(&self.input)?;
                let reader = BufReader::new(&path);
                let entries = read_entries(&self.format, config, reader)?;
                self.results = Some(entries.clone());
                entries
            }
        };
        let filter = new_searchfn(&search);
        Ok(entries.into_iter().filter(|e| filter(e)).collect())
    }
}

struct Plugin {
    name: String,
    args: Vec<String>,
    format: Format,
    results: Option<Vec<Entry>>,
    command: Cmd,
    cache_thread: Option<std::thread::JoinHandle<()>>,
}

impl Plugin {
    pub fn new(name: String, config: &PluginConfig) -> Result<Self> {
        let args: Vec<String> = config
            .exec
            .iter()
            .map(|s| shellexpand::tilde(s).to_string())
            .collect();
        Ok(Self {
            name,
            args,
            format: config.format.clone(),
            results: None,
            command: Cmd::NotStarted,
            cache_thread: None,
        })
    }

    fn read(&mut self, config: &mut Config) -> Result<()> {
        let command = self.command.child();
        let stdout = command
            .stdout
            .as_mut()
            .ok_or_else(|| RMenuError::CommandError(None))?;
        let reader = BufReader::new(stdout);
        let entries = read_entries(&self.format, config, reader)?;
        self.results = Some(entries);
        Ok(())
    }

    pub fn memory_search(&mut self, search: &Search) -> Result<Vec<Entry>> {
        let results = self.results.as_ref().expect("results should be set");
        let filter = new_searchfn(&search);
        return Ok(results.iter().filter(|e| filter(e)).cloned().collect());
    }

    pub fn write_cache(&mut self, config: &mut Config, query: &str) {
        if query.is_empty() {
            let plugin = config
                .plugins
                .get(&self.name)
                .cloned()
                .expect("missing plugin config");
            let name = self.name.to_owned();
            let results = self.results.clone().expect("results should be set");
            self.cache_thread = Some(std::thread::spawn(move || match crate::cache::write_cache(
                &name, &plugin, &results,
            ) {
                Ok(_) => {}
                Err(err) => log::error!("cache write error: {err:?}"),
            }));
        }
    }

    pub fn search(&mut self, config: &mut Config, query: &str) -> Result<Vec<Entry>> {
        let search = new_search(query, &config);
        match self.command {
            Cmd::Started(_) => {}
            Cmd::Skipped => return self.memory_search(&search),
            Cmd::NotStarted => {
                // check cache if not already loaded
                if self.results.is_none() {
                    let plugin = config
                        .plugins
                        .get(&self.name)
                        .expect("missing plugin config");
                    match crate::cache::read_cache(&self.name, &plugin) {
                        Err(err) => log::error!("cache read failed: {err:?}"),
                        Ok(cached) => {
                            log::info!(
                                "{:?} entries read from cache for plugin {:?}",
                                cached.len(),
                                self.name
                            );
                            self.command = Cmd::Skipped;
                            self.results = Some(cached);
                            return self.memory_search(&search);
                        }
                    }
                }
                // spawn command for later processing
                let main = self
                    .args
                    .get(0)
                    .ok_or_else(|| RMenuError::InvalidPlugin(self.name.to_owned()))?;

                let mut cmd = Command::new(main);
                #[cfg(target_os = "windows")]
                let command = cmd.creation_flags(0x08000000).args(&self.args[1..]);
                #[cfg(not(target_os = "windows"))]
                let command = cmd.args(&self.args[1..]);

                self.command = Cmd::Started(
                    command
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()?,
                );
            }
        }
        // handle case where command has exited and returned final results
        log::debug!("plugin {:?} searching {search:?}", self.name);
        let command = self.command.child();
        let status = command.try_wait()?;
        if let Some(status) = status {
            if !status.success() {
                return Err(RMenuError::CommandError(Some(status)));
            }
            if self.results.is_none() {
                self.read(config)?;
                self.write_cache(config, query);
            }
            return self.memory_search(&search);
        }
        // send search message to program
        log::debug!("sending search message to plugin {:?}", self.name);
        let mut message = serde_json::to_vec(&search)?;
        message.push(b'\n');
        let stdin = command
            .stdin
            .as_mut()
            .ok_or_else(|| RMenuError::CommandError(None))?;
        stdin.write(&message)?;
        // read response and return results
        log::debug!("reading replies from plugin {:?}", self.name);
        self.read(config)?;
        self.write_cache(config, query);
        Ok(self
            .results
            .as_ref()
            .expect("results should be set")
            .clone())
    }
}
