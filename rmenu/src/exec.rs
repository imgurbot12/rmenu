//! Execution Implementation for Entry Actions
use std::os::unix::process::CommandExt;
use std::process::Command;

use rmenu_plugin::Action;

pub fn execute(action: &Action) {
    log::info!("executing: {} {:?}", action.name, action.exec);
    let args = match shell_words::split(&action.exec) {
        Ok(args) => args,
        Err(err) => panic!("{:?} invalid command {err}", action.exec),
    };
    let err = Command::new(&args[0]).args(&args[1..]).exec();
    panic!("Command Error: {err:?}");
}
