use clap::{Parser, Subcommand};
use rmenu_plugin::Entry;

mod pulse;

#[derive(Debug, Subcommand)]
pub enum Commands {
    ListSinks,
    SetDefaultSink { sink: u32 },
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

fn main() -> Result<(), pulse::PulseError> {
    let cli = Cli::parse();
    let exe = std::env::current_exe()?.to_str().unwrap().to_string();

    let command = cli.command.unwrap_or(Commands::ListSinks);
    match command {
        Commands::ListSinks => {
            let sinks = pulse::get_sinks()?;
            for sink in sinks {
                let star = sink.default.then(|| "* ").unwrap_or("");
                let desc = format!("{star}{}", sink.description);
                let exec = format!("{exe} set-default-sink {}", sink.index);
                let entry = Entry::new(&desc, &exec, None);
                println!("{}", serde_json::to_string(&entry).unwrap());
            }
        }
        Commands::SetDefaultSink { sink } => {
            let sinkstr = format!("{sink}");
            pulse::set_default_sink(&sinkstr)?;
        }
    }

    Ok(())
}
