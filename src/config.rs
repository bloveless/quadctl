use std::{collections::HashMap, fs, path::PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unable to read inventory.toml: {0}")]
    UnableToReadInventory(std::io::Error),
    #[error("unable to read file {0}: {1}")]
    UnableToReadFile(String, std::io::Error),
    #[error("toml parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("unknown dependency for {0}: {1}")]
    DependencyUnknown(String, String),
}

#[derive(Debug, Deserialize)]
pub struct Quadlet {
    pub name: String,
    pub file: String,
    pub depends_on: Option<Vec<String>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Node {
    pub name: String,
    pub host: String,
    pub user: String,
    pub ssh_key: String,
    pub remote_path: String,
    pub tags: Vec<String>,
    pub quadlets: Vec<Quadlet>,
}

#[derive(Debug, Deserialize)]
pub struct Inventory {
    pub nodes: Vec<Node>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Config {
    pub config_dir: PathBuf,
    pub inventory: Inventory,
    pub node_hashes: HashMap<String, HashMap<String, String>>,
}

pub fn load_config(config_dir: &PathBuf) -> Result<Config, Error> {
    let inventory_file = std::fs::read_to_string(config_dir.join("inventory.toml"))
        .map_err(Error::UnableToReadInventory)?;
    let inventory: Inventory = toml::from_str(&inventory_file)?;
    let node_hashes = compute_hashes(config_dir, &inventory)?;
    Ok(Config {
        config_dir: config_dir.clone(),
        inventory,
        node_hashes,
    })
}

fn compute_hashes(
    config_dir: &PathBuf,
    inventory: &Inventory,
) -> Result<HashMap<String, HashMap<String, String>>, Error> {
    let mut node_hashes = std::collections::HashMap::new();
    for node in &inventory.nodes {
        let mut quadlet_hashes = std::collections::HashMap::new();
        for q in &node.quadlets {
            compute_hash(node, &mut quadlet_hashes, config_dir, q)?;
        }
        node_hashes.insert(node.name.clone(), quadlet_hashes);
    }
    Ok(node_hashes)
}

fn compute_hash(
    node: &Node,
    quadlet_hashes: &mut std::collections::HashMap<String, String>,
    config_dir: &PathBuf,
    q: &Quadlet,
) -> Result<(), Error> {
    if quadlet_hashes.contains_key(&q.name) {
        return Ok(());
    }

    let quad_file = config_dir.join(&q.file);
    let hash = hash_file(&quad_file)?;
    quadlet_hashes.insert(q.name.clone(), hash.clone());

    // DFS for computing hashes of dependencies
    if let Some(deps) = &q.depends_on {
        let mut hashes = Vec::new();
        hashes.push(hash);

        for dep in deps {
            if quadlet_hashes.contains_key(dep) {
                hashes.push(quadlet_hashes[dep].clone());
            } else {
                let dep_quad = node.quadlets.iter().find(|q| q.name == *dep);
                if let Some(dep_quad) = dep_quad {
                    compute_hash(node, quadlet_hashes, config_dir, dep_quad)?;
                    hashes.push(quadlet_hashes[dep].clone());
                } else {
                    return Err(Error::DependencyUnknown(q.name.clone(), dep.to_owned()));
                }
            }
        }

        quadlet_hashes.insert(q.name.clone(), hex::encode(Sha256::digest(hashes.concat())));
    }

    Ok(())
}

fn hash_file(path: &PathBuf) -> Result<String, Error> {
    let file_contents = fs::read_to_string(path)
        .map_err(|e| Error::UnableToReadFile(path.to_string_lossy().into(), e))?;

    let mut hasher = Sha256::new();
    hasher.update(file_contents.as_bytes());
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_inventory() {
        let config = load_config(&PathBuf::from("tests/data/missing-inventory"));
        assert!(matches!(config, Err(Error::UnableToReadInventory(_))));
    }

    #[test]
    fn test_missing_dependency() {
        let config = load_config(&PathBuf::from("tests/data/missing-dependency"));
        let Error::DependencyUnknown(parent, dep) = config.unwrap_err() else {
            panic!("Expected Error::DependencyUnknown, but got a different error");
        };
        assert_eq!(parent, "parent-container");
        assert_eq!(dep, "parent-volume-doesnt-exist");
    }

    #[test]
    fn test_nominal() {
        let config = load_config(&PathBuf::from("tests/data/nominal"))
            .expect("failed to load nominal config");
        assert_eq!(config.config_dir.to_string_lossy(), "tests/data/nominal");
        assert_eq!(config.inventory.nodes.len(), 1);
        let expected: HashMap<String, HashMap<String, String>> = HashMap::from([(
            "node01".into(),
            HashMap::from([
                (
                    "parent-container".into(),
                    "26ff8bf4cf7f7e09d292f056edf83850b6f8004577a116512b6b860c33188e27".into(),
                ),
                (
                    "parent-network".into(),
                    "079aef93a32496f93a1b34db5fd9ac23289f130e97a0b6b75983ef066b2b008b".into(),
                ),
                (
                    "parent-volume".into(),
                    "4e8dd3bfb2135d432cbbc00fb4da890319059df8f583fc8e324d73ba1e9d2b51".into(),
                ),
            ]),
        )]);

        assert_eq!(config.node_hashes, expected);
    }
}
