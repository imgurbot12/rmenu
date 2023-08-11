use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};

use thiserror::Error;

static PACTL: &'static str = "pactl";

#[derive(Debug, Error)]
pub enum PulseError {
    #[error("Invalid Status")]
    InvalidStatus(ExitStatus),
    #[error("Cannot Read Command Output")]
    InvalidStdout,
    #[error("Invalid Output Encoding")]
    InvalidEncoding(#[from] std::string::FromUtf8Error),
    #[error("Unexepcted Command Output")]
    UnexepctedOutput(String),
    #[error("Command Error")]
    CommandError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct Sink {
    pub index: u32,
    pub name: String,
    pub description: String,
    pub default: bool,
}

/// Retrieve Default Sink
pub fn get_default_sink() -> Result<String, PulseError> {
    let output = Command::new(PACTL).arg("get-default-sink").output()?;
    if !output.status.success() {
        return Err(PulseError::InvalidStatus(output.status));
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Set Default PACTL Sink
pub fn set_default_sink(name: &str) -> Result<(), PulseError> {
    let status = Command::new(PACTL)
        .args(["set-default-sink", name])
        .status()?;
    if !status.success() {
        return Err(PulseError::InvalidStatus(status));
    }
    Ok(())
}

/// Retrieve PulseAudio Sinks From PACTL
pub fn get_sinks() -> Result<Vec<Sink>, PulseError> {
    // retrieve default-sink
    let default = get_default_sink()?;
    // spawn command and begin reading
    let mut command = Command::new(PACTL)
        .args(["list", "sinks"])
        .stdout(Stdio::piped())
        .spawn()?;
    let stdout = command.stdout.as_mut().ok_or(PulseError::InvalidStdout)?;
    // collect output into only important lines
    let mut lines = vec![];
    for line in BufReader::new(stdout).lines().filter_map(|l| l.ok()) {
        let line = line.trim_start();
        if line.starts_with("Sink ")
            || line.starts_with("Name: ")
            || line.starts_with("Description: ")
        {
            lines.push(line.to_owned());
        }
    }
    // ensure status after command completion
    let status = command.wait()?;
    if !status.success() {
        return Err(PulseError::InvalidStatus(status));
    }
    // ensure number of lines matches expected
    if lines.len() == 0 || lines.len() % 3 != 0 {
        return Err(PulseError::UnexepctedOutput(lines.join("\n")));
    }
    // parse details into chunks and generate sinks
    let mut sinks = vec![];
    for chunk in lines.chunks(3) {
        let (_, idx) = chunk[0]
            .split_once("#")
            .ok_or_else(|| PulseError::UnexepctedOutput(chunk[0].to_owned()))?;
        let (_, name) = chunk[1]
            .split_once(" ")
            .ok_or_else(|| PulseError::UnexepctedOutput(chunk[1].to_owned()))?;
        let (_, desc) = chunk[2]
            .split_once(" ")
            .ok_or_else(|| PulseError::UnexepctedOutput(chunk[2].to_owned()))?;
        sinks.push(Sink {
            index: idx.parse().unwrap(),
            name: name.to_owned(),
            description: desc.to_owned(),
            default: name == default,
        });
    }
    Ok(sinks)
}
