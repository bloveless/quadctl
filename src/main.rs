use clap::{Parser, Subcommand};
use quadctl::{config, remote_check, ssh};
use std::path::{Path, PathBuf};

use crate::ssh::SshRemoteConnector;

#[derive(Parser)]
#[command(name = "quadctl", about = "Quadlet control tool")]
struct Cli {
    #[arg(short, long, default_value = ".")]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Checks that known files in inventory match the simple remote hash
    RemoteCheck,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config_dir = Path::new("manifests").to_path_buf();

    let config = match config::load_config(&config_dir) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("unable to load config: {}", e);
            return Err(e.into());
        }
    };

    match &cli.command {
        Some(Commands::RemoteCheck) => {
            let remote_connector = SshRemoteConnector {};
            remote_check::remote_check(&remote_connector, &config).await?;
        }
        None => {
            for (node, quadlet_hashes) in &config.node_hashes {
                let hash_toml = toml::to_string(&quadlet_hashes).unwrap();
                println!("node: {}", node);
                println!("{}", hash_toml);
                // Now that we have the hashes we need to pull the remote hash file into memory and compare the hashes
                // to see if anything has changed.
                //
                // We need to push up all changed files.
                //
                // If any container files have changes we need to restart them using `systemctl --user restart <container>`
                //
                // Finally we need to push up a new hash file to the remote server.
            }
        }
    };

    Ok(())
}
