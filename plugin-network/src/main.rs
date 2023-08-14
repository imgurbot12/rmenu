use anyhow::{anyhow, Result};

use clap::{Parser, Subcommand};
use rmenu_plugin::Entry;

mod gui;
mod network;

#[derive(Debug, Subcommand)]
pub enum Commands {
    ListAccessPoints,
    Connect { ssid: String, timeout: Option<u32> },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

/// Get Signal Bars based on a signal-value
fn get_bars(signal: u8) -> String {
    if signal >= 80 {
        return "▂▄▆█".to_owned();
    }
    if signal > 50 {
        return "▂▄▆_".to_owned();
    }
    if signal > 30 {
        return "▂▄__".to_owned();
    }
    "▂___".to_owned()
}

/// List AccessPoints using NetworkManager
async fn list_aps() -> Result<()> {
    let exe = std::env::current_exe()?.to_str().unwrap().to_string();
    // spawn manager and complete scan (if nessesary)
    let manager = network::Manager::new().await?;
    if !manager.scanned_recently() {
        log::info!("Scanning for Access-Points...");
        manager.scan_wifi().await?;
    }
    // retrive access-points and print as entries
    for ap in manager.access_points() {
        let star = ap.is_active.then(|| " *").unwrap_or("");
        let bars = get_bars(ap.signal);
        let desc = format!("{bars} {}{star}", ap.ssid);
        let exec = format!("{exe} connect {:?}", ap.ssid);
        let entry = Entry::new(&desc, &exec, None);
        println!("{}", serde_json::to_string(&entry).unwrap());
    }
    Ok(())
}

/// Attempt to Connect to the Specified SSID (If Connection Already Exists)
async fn try_connect_ap(ssid: String, timeout: Option<u32>) -> Result<bool> {
    // spawn manager and complete scan (if nessesary)
    let mut manager = network::Manager::new().await?;
    if let Some(timeout) = timeout {
        manager = manager.with_timeout(timeout)
    }
    if !manager.scanned_recently() {
        log::info!("Scanning for Access-Points...");
        manager.scan_wifi().await?;
    }
    // attempt to find access-point
    let access_point = manager
        .access_points()
        .into_iter()
        .find(|ap| ap.ssid == ssid)
        .ok_or_else(|| anyhow!("Unable to find Access-Point: {ssid:?}"))?;
    // if connection already exists, try to connect
    if access_point.connection.is_some() {
        log::info!("Attempting Connection to {ssid:?} w/o Password");
        match manager.connect(&access_point, None).await {
            Err(err) => log::info!("Connection Failed: {err:?}"),
            Ok(_) => {
                log::info!("Connection Successful!");
                return Ok(true);
            }
        };
    }
    Ok(false)
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let context = glib::MainContext::default();
    let command = cli.command.unwrap_or(Commands::ListAccessPoints);
    match command {
        Commands::ListAccessPoints => context.block_on(list_aps())?,
        Commands::Connect { ssid, timeout } => {
            let connected = context.block_on(try_connect_ap(ssid.clone(), timeout))?;
            if !connected {
                log::info!("Spawning GUI to complete AP Login");
                gui::run_app(&ssid, timeout.unwrap_or(30));
            }
        }
    }
    Ok(())
}
