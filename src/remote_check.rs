use crate::{
    config::{Config, Node, Quadlet},
    ssh::{RemoteConnector, RemoteExecutor},
};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to execute command on node: {0}")]
    OpenSsh(#[from] openssh::Error),
    #[error("failed to connect to node: {0}")]
    Ssh(#[from] crate::ssh::Error),
    #[error("failed to get remote hash: {0}")]
    RemoteHash(String),
    #[error("failed to read local quadlet: {0}")]
    Io(#[from] std::io::Error),
}

pub async fn remote_check<C: RemoteConnector>(
    remote_connector: &C,
    config: &Config,
) -> Result<(), Error> {
    for node in &config.inventory.nodes {
        let session = remote_connector.connect(&node.host, &node.user).await?;
        for quadlet in &node.quadlets {
            let remote_hash = get_remote_hash(node, quadlet, &session).await?;
            let local_quadlet_path = config.config_dir.join(&node.local_path).join(&quadlet.file);
            let local_quadlet_data = tokio::fs::read(&local_quadlet_path).await?;
            let local_quadlet_hash = hex::encode(Sha256::digest(local_quadlet_data));
            println!(
                "Checking quadlet {}\nremote hash = {}\nlocal hash  = {}",
                quadlet.name, remote_hash, local_quadlet_hash
            );
            if remote_hash != local_quadlet_hash {
                println!("Hashes didn't match");
            }
            println!("--\n")
        }
    }

    Ok(())
}

async fn get_remote_hash<E: RemoteExecutor>(
    node: &Node,
    quadlet: &Quadlet,
    client: &E,
) -> Result<String, Error> {
    let quadlet_path = node.remote_path.join(&quadlet.file);
    let Some(quadlet_path) = quadlet_path.to_str() else {
        return Err(Error::RemoteHash(format!(
            "Skipping remote quadlet: {:?} (path is not valid UTF-8)",
            quadlet.file
        )));
    };
    println!("Checking remote quadlet: {}", quadlet_path);
    let sum = client.run_command("sha256sum", &[quadlet_path]).await?;
    if !sum.success {
        return Err(Error::RemoteHash(format!(
            "failed to get remote hash for quadlet {}: {}",
            quadlet_path,
            sum.stderr.trim()
        )));
    }
    let (hash, _) = sum.stdout.split_once(' ').ok_or_else(|| {
        Error::RemoteHash(format!(
            "failed to parse remote hash for quadlet {}: {}",
            quadlet_path,
            sum.stdout.trim()
        ))
    })?;

    Ok(hash.trim().to_string())
}
