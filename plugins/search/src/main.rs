use std::io::{BufRead, BufReader, Read};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rmenu_plugin::{Entry, Message, Search};

mod bang;
mod pattern;

use crate::bang::Bang;
use crate::pattern::Patterns;

const DEFAULT_BANG: &'static str = "!brave";

#[cfg(target_os = "windows")]
const CMD_PREFIX: &'static str = "cmd /K ";
#[cfg(target_family = "unix")]
const CMD_PREFIX: &'static str = "";

#[cfg(target_os = "windows")]
const START_COMMAND: &'static str = "start";
#[cfg(target_family = "unix")]
const START_COMMAND: &'static str = "xdg-open";

#[derive(Debug, Subcommand)]
pub enum Commands {
    Search,
    Copy { contents: String },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

fn send_entry(entry: &Entry) {
    let message = serde_json::to_string(&entry).expect("invalid entry");
    println!("{message}");
}

fn read_search<T: Read>(reader: &mut BufReader<T>) -> Option<Search> {
    let mut buf = String::new();
    match reader.read_line(&mut buf) {
        Ok(_) => {}
        Err(err) => {
            log::error!("failed to read stdin: {err:?}");
            return None;
        }
    }
    buf = buf.trim().to_owned();
    if buf.is_empty() {
        log::debug!("message empty. breaking loop.");
        return None;
    }
    serde_json::from_str(&buf).ok()
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Search);
    match command {
        Commands::Copy { contents } => {
            rmenu_plugin::extra::clipboard::copy_and_exit(&contents).map_err(anyhow::Error::msg)
        }
        Commands::Search => search(),
    }
}

fn search() -> Result<()> {
    let exe = rmenu_plugin::extra::get_exe();
    let bangs = Bang::bangs();
    let rgx = regex::RegexBuilder::new(r"!\w+")
        .build()
        .context("bang regex failed to compile")?;

    // configure fend
    let mut fend = fend_core::Context::new();
    fend.set_random_u32_fn(|| rand::random());

    // configure additiona patterns
    let patterns = Patterns::new();

    // prepare stdin reader
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin);

    loop {
        // read search message from stdin
        let Some(search) = read_search(&mut reader) else {
            continue;
        };

        // retrieve bang based on search
        let search_bang = rgx.find(&search.search).map(|m| m.as_str());

        // check if search can be processed by fend
        if let Ok(result) = fend_core::evaluate(&search.search, &mut fend) {
            let calc = result.get_main_result();
            if !calc.is_empty() {
                let name = format!(" = {calc}");
                let action = format!("{CMD_PREFIX}{exe} copy {calc:?}");
                let entry = Entry::new(&name, &action, None);
                send_entry(&entry);
            }
        }

        // retrieve bang based on search
        let search_bang = search_bang.unwrap_or(DEFAULT_BANG);
        let bang = match bangs.get(&search_bang[1..]) {
            Some(bang) => bang,
            None => {
                log::warn!("invalid bang: {search_bang:?}");
                bangs
                    .get(&DEFAULT_BANG[1..])
                    .context("default bang missing")?
            }
        };

        // remove bang from search string
        let query = search.search.replace(search_bang, "");
        let query = query.trim();

        // attempt to match available patterns
        if let Some(entries) = patterns.try_match(query, Some(bang)) {
            for entry in entries.iter() {
                send_entry(&entry);
            }
        }

        let name = format!("{} - {query}", bang.name);
        let escaped = url_escape::encode_component(query).to_string();

        let url = bang.url.replace(r"{{{s}}}", &escaped);
        let action = format!("{CMD_PREFIX}{START_COMMAND} {url:?}",);
        let entry = Entry::new(&name, &action, None);
        send_entry(&entry);

        let stop = Message::Stop;
        let message = serde_json::to_string(&stop).context("invalid stop message")?;
        println!("{message}");
    }
}
