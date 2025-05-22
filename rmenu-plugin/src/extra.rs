///! Useful and Reused RMenu Plugin Extensions

/// Shortcut for Getting Current Executable of Self
#[inline]
pub fn get_exe() -> String {
    std::env::current_exe()
        .expect("failed to discover self")
        .to_str()
        .expect("invalid executable")
        .to_string()
}

/// Generate Open Command to Access Resource
#[cfg(target_os = "windows")]
pub fn open_command(resource: impl std::fmt::Debug) -> String {
    format!("cmd /K start {resource:?}")
}

/// Generate Open Command to Access Resource
#[cfg(target_family = "unix")]
pub fn open_command(resource: impl std::fmt::Debug) -> String {
    format!("xdg-open {resource:?}")
}

#[cfg(target_os = "windows")]
pub mod clipboard {
    use clipboard_win::{formats, Clipboard, Setter};

    /// Copy Contents to Clipboard and Exit
    pub fn copy_and_exit(content: &str) -> Result<(), String> {
        let _clip =
            Clipboard::new_attempts(10).map_err(|e| format!("failed to open clipboard {e:?}"))?;
        formats::Unicode
            .write_clipboard(&content)
            .map_err(|e| format!("failed to write clipboard: {e:?}"))?;
        std::process::exit(0);
    }
}

#[cfg(target_family = "unix")]
pub mod clipboard {
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    /// Retrieve Avaialble Copy Command from System
    fn get_clipboard_command() -> Result<Command, String> {
        if let Ok(path) = which::which("wl-copy") {
            Ok(std::process::Command::new(path))
        } else if let Ok(path) = which::which("xsel") {
            let mut cmd = std::process::Command::new(path);
            cmd.arg("-ib");
            Ok(cmd)
        } else if let Ok(path) = which::which("xclip") {
            let mut cmd = std::process::Command::new(path);
            cmd.arg("-selection");
            cmd.arg("c");
            Ok(cmd)
        } else {
            Err("no copy command available!".to_owned())
        }
    }

    /// Copy Contents to Clipboard and Exit
    pub fn copy_and_exit(content: &str) -> Result<(), String> {
        let command = get_clipboard_command()?;
        command
            .arg(content)
            .exec()
            .map_err(|e| format!("exec failed: {e:?}"))
    }
}
