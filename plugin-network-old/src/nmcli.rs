//! Command Bindings to NetworkManager-CLI
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Execution Error")]
    ExecError(#[from] std::io::Error),
    #[error("No Stdout")]
    NoStdout,
    #[error("Unexpected Command Output")]
    UnexpectedOutput(String),
    #[error("Invalid Integer")]
    ParseError(#[from] std::num::ParseIntError),
    #[error("Unexpected ExitCode")]
    StatusCode(ExitStatus),
}

#[derive(Debug)]
pub struct Network {
    pub in_use: bool,
    pub ssid: String,
    pub rate: u32,
    pub signal: u32,
    pub bars: String,
    pub security: String,
}

/// Retrieve Key from HashMap of Fields
#[inline]
fn get_key<'a>(
    m: &'a BTreeMap<&str, &str>,
    key: &'a str,
    line: &'a str,
) -> Result<&'a &'a str, NetworkError> {
    m.get(key)
        .ok_or_else(|| NetworkError::UnexpectedOutput(line.to_owned()))
}

/// Complete Network Scan and Retrieve Available Networks
pub fn scan() -> Result<Vec<Network>, NetworkError> {
    // spawn command
    let mut command = Command::new("nmcli")
        .args(["dev", "wifi", "list", "--rescan", "auto"])
        .stdout(Stdio::piped())
        .spawn()?;
    // retrieve stdout
    let stdout = command.stdout.as_mut().ok_or(NetworkError::NoStdout)?;
    let mut reader = BufReader::new(stdout);
    // parse initial header
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let headers: Vec<&str> = line.trim().split(" ").filter(|i| i.len() > 0).collect();
    // parse body elements
    let mut netmap = BTreeMap::<String, Network>::new();
    for line in reader.lines().filter_map(|l| l.ok()) {
        // parse into key/value pairs
        let mut values = vec![];
        let mut state = line.as_str();
        for _ in 0..headers.len() {
            let (value, rest) = state.trim().split_once("  ").unwrap_or((state, ""));
            if value.is_empty() {
                break;
            }
            values.push(value);
            state = rest;
        }
        if values.len() < headers.len() {
            values.insert(0, "");
        }
        let fields: BTreeMap<&str, &str> =
            headers.iter().cloned().zip(values.into_iter()).collect();
        // retrieve required fields
        let bars = get_key(&fields, "BARS", &line).map(|s| s.to_string())?;
        let ssid = get_key(&fields, "SSID", &line).map(|s| s.to_string())?;
        let in_use = get_key(&fields, "IN-USE", &line).map(|s| s.len() > 0)?;
        let security = get_key(&fields, "SECURITY", &line).map(|s| s.to_string())?;
        let rate = get_key(&fields, "RATE", &line)?
            .split_once(" ")
            .ok_or_else(|| NetworkError::UnexpectedOutput(line.to_owned()))?
            .0
            .parse()?;
        let signal = get_key(&fields, "SIGNAL", &line)?.parse()?;
        let network = Network {
            in_use,
            bars,
            rate,
            ssid,
            signal,
            security,
        };
        // skip entry if the current entry has better rate/signal
        if let Some(other) = netmap.get(&network.ssid) {
            if other.rate > network.rate && other.signal > network.signal {
                continue;
            }
        }
        netmap.insert(network.ssid.to_owned(), network);
    }
    // check status of response
    let status = command.wait()?;
    if !status.success() {
        return Err(NetworkError::StatusCode(status));
    }
    // collect networks into a vector and sort by signal
    let mut networks: Vec<Network> = netmap.into_values().collect();
    networks.sort_by_key(|n| n.rate * n.signal);
    networks.reverse();
    Ok(networks)
}
