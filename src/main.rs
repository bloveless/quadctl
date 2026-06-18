use std::path::Path;

mod config;

fn main() {
    let config_dir = Path::new("manifests").to_path_buf();

    let config = match config::load_config(&config_dir) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("unable to load config: {}", e);
            return;
        }
    };

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
