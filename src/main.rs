use clap::{Parser, Subcommand};
use nd2_rs::{Nd2File, Result};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nd2-rs")]
#[command(version, about = "Read Nikon ND2 microscopy files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print file metadata as JSON
    Info {
        /// Path to the ND2 file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { file } => {
            let mut nd2 = Nd2File::open(&file)?;
            let version = nd2.version();
            let attributes = nd2.attributes()?.clone();
            let text_info = nd2.text_info()?.clone();
            let experiment = nd2.experiment()?.clone();

            let output = serde_json::json!({
                "version": { "major": version.0, "minor": version.1 },
                "attributes": attributes,
                "text_info": text_info,
                "experiment": experiment,
            });
            println!("{}", serde_json::to_string_pretty(&output).expect("JSON"));
        }
    }

    Ok(())
}
