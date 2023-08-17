///! Functions to Build PowerMenu Actions
use std::collections::BTreeMap;
use std::env;
use std::io::Write;
use std::process;

use rmenu_plugin::{Action, Entry};
use tempfile::NamedTempFile;

use crate::Command;

//TODO: dynamically determine actions based on OS/Desktop/etc...

/// Ordered Map of Configured Actions
pub type Actions = BTreeMap<Command, Entry>;

/// Generate Confirmation for Specific Command
fn build_confirm(command: Command, actions: &Actions) -> Vec<Entry> {
    let entry = actions.get(&command).expect("Invalid Command");
    let cancel = format!("echo '{command} Cancelled'");
    vec![
        Entry {
            name: "Cancel".to_owned(),
            actions: vec![Action::new(&cancel)],
            comment: None,
            icon: None,
            icon_alt: Some("".to_owned()),
        },
        Entry {
            name: "Confirm".to_owned(),
            actions: entry.actions.to_owned(),
            comment: None,
            icon: None,
            icon_alt: Some("".to_owned()),
        },
    ]
}

/// Generate Confirm Actions and Run Rmenu
pub fn confirm(command: Command, actions: &Actions) {
    let rmenu = env::var("RMENU").unwrap_or_else(|_| "rmenu".to_owned());
    let entries = build_confirm(command, actions);
    // write to temporary file
    let mut f = NamedTempFile::new().expect("Failed to Open Temporary File");
    for entry in entries {
        let json = serde_json::to_string(&entry).expect("Failed Serde Serialize");
        write!(f, "{json}\n").expect("Failed Write");
    }
    // run command to read from temporary file
    let path = f.path().to_str().expect("Invalid Temporary File Path");
    let mut command = process::Command::new(rmenu)
        .args(["-i", path])
        .spawn()
        .expect("Command Spawn Failed");
    let status = command.wait().expect("Command Wait Failed");
    if !status.success() {
        panic!("Command Failed: {status:?}");
    }
}

/// Calculate and Generate PowerMenu Actions
pub fn list_actions() -> Actions {
    let mut actions = BTreeMap::new();
    actions.extend(vec![
        (
            Command::Shutdown,
            Entry {
                name: "Shut Down".to_owned(),
                actions: vec![Action::new("systemctl poweroff")],
                comment: None,
                icon: None,
                icon_alt: Some("⏻".to_owned()),
            },
        ),
        (
            Command::Reboot,
            Entry {
                name: "Reboot".to_owned(),
                actions: vec![Action::new("systemctl reboot")],
                comment: None,
                icon: None,
                icon_alt: Some(" ".to_owned()),
            },
        ),
        (
            Command::Suspend,
            Entry {
                name: "Suspend".to_owned(),
                actions: vec![Action::new("systemctl suspend")],
                comment: None,
                icon: None,
                icon_alt: Some("⏾".to_owned()),
            },
        ),
        (
            Command::Logout,
            Entry {
                name: "Log Out".to_owned(),
                actions: vec![Action::new("sway exit")],
                comment: None,
                icon: None,
                icon_alt: Some("".to_owned()),
            },
        ),
    ]);
    actions
}
