/// RMenu Plugin Result Entry Server
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, ExitStatus, Stdio};

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
}

pub struct Plugin {
    name: String,
    args: Vec<String>,
    format: Format,
    results: Option<Vec<Entry>>,
    command: Cmd,
    cache_thread: Option<std::thread::JoinHandle<()>>,
}

impl Plugin {
    pub fn start(name: String, config: &PluginConfig) -> Result<Self> {
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
        let mut entries = vec![];
        let reader = BufReader::new(stdout);
        for line in reader.lines().filter_map(|l| l.ok()) {
            match &self.format {
                Format::DMenu => entries.push(Entry::echo(line.trim(), None)),
                Format::Json => {
                    let msg: Message = serde_json::from_str(&line)?;
                    match msg {
                        Message::Stop => break,
                        Message::Entry(entry) => entries.push(entry),
                        Message::Options(options) => config
                            .update(&options)
                            .map_err(|s| RMenuError::InvalidKeybind(s))?,
                    }
                }
            }
        }
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
        let search = Search {
            search: query.to_owned(),
            is_regex: config.search.use_regex,
            ignore_case: config.search.ignore_case,
        };
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
                self.command = Cmd::Started(
                    Command::new(main)
                        .args(&self.args[1..])
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

pub struct Server {
    order: Vec<String>,
    active: Vec<String>,
    plugins: HashMap<String, Plugin>,
}

impl Server {
    pub fn start(config: &mut Config, run: Vec<String>, mut active: Vec<String>) -> Result<Self> {
        let mut plugins = HashMap::new();
        if active.is_empty() {
            let mode = run.get(0).expect("no active plugins").clone();
            log::warn!("no mode specified. defaulting to {mode:?}");
            active.push(mode);
        }
        for name in run.iter() {
            let cfg = config
                .plugins
                .get(name)
                .cloned()
                .ok_or_else(|| RMenuError::NoSuchPlugin(name.to_owned()))?;
            if let Some(options) = cfg.options.as_ref() {
                config
                    .update(options)
                    .map_err(|e| RMenuError::InvalidKeybind(e))?;
            }
            let plugin = Plugin::start(name.to_owned(), &cfg)?;
            plugins.insert(name.to_owned(), plugin);
        }
        Ok(Self {
            active,
            plugins,
            order: run,
        })
    }

    pub fn search(&mut self, config: &mut Config, query: &str) -> Result<Vec<Entry>> {
        let mut results = vec![];
        for name in self.active.iter() {
            let plugin = self.plugins.get_mut(name).expect("plugin missing");
            let entries = plugin.search(config, query)?;
            results.extend(entries);
        }
        Ok(results)
    }

    pub fn placeholder(&self, config: &Config) -> String {
        let mode = self.active.last().expect("no active plugins");
        let plugin = config.plugins.get(mode).expect("invalid plugin");
        config
            .search
            .placeholder
            .clone()
            .or(plugin.placeholder.clone())
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
        for plugin in self.plugins.values_mut() {
            if let Some(thread) = plugin.cache_thread.take() {
                threads.push(thread);
            }
        }
        log::debug!("cleaning up {} threads", threads.len());
        while !threads.is_empty() {
            let thread = threads.pop().unwrap();
            let _ = thread.join();
        }
    }
}
