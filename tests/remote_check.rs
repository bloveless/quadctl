use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use quadctl::{
    config::{Error as ConfigError, load_config},
    remote_check::{self, Error as RemoteCheckError},
    ssh::{self, RemoteConnector, RemoteExecutor, RunCommandResult},
};
use thiserror::Error;

struct FakeConnector {
    responses: HashMap<String, RunCommandResult>,
}

impl FakeConnector {
    fn new(responses: HashMap<String, RunCommandResult>) -> Self {
        Self { responses }
    }
}

impl RemoteConnector for FakeConnector {
    type Client = FakeClient;

    async fn connect(&self, host: &str, user: &str) -> Result<Self::Client, ssh::Error> {
        Ok(FakeClient {
            responses: self.responses.clone(),
        })
    }
}

struct FakeClient {
    responses: HashMap<String, RunCommandResult>,
}

impl RemoteExecutor for FakeClient {
    async fn run_command(
        &self,
        command: &str,
        args: &[&str],
    ) -> Result<RunCommandResult, quadctl::ssh::Error> {
        if !self.responses.contains_key(args[0]) {
            return Ok(RunCommandResult {
                success: false,
                stdout: "".into(),
                stderr: "".into(),
            });
        }
        Ok(self.responses[args[0]].clone())
    }
}

#[derive(Error, Debug)]
enum Error {
    #[error("config error {0}")]
    Config(#[from] ConfigError),
    #[error("remote check error {0}")]
    RemoteCheck(#[from] RemoteCheckError),
}

#[tokio::test]
async fn remote_check_nominal() -> Result<(), Error> {
    let config = load_config(&PathBuf::from("tests/data/nominal"))?;
    let connector = FakeConnector::new(HashMap::from([
        (
            "/etc/containers/systemd/parent.container".into(),
            RunCommandResult {
                success: true,
                stdout: "08e9b70eecceb5a0728d756e1a6f5f9dab96207665e2ce434371adf5462bf815 /etc/containers/systemd/parent.container".into(),
                stderr: "".into(),
            },
        ),
        (
            "/etc/containers/systemd/parent.network".into(),
            RunCommandResult {
                success: true,
                stdout: "079aef93a32496f93a1b34db5fd9ac23289f130e97a0b6b75983ef066b2b008b /etc/containers/systemd/parent.network".into(),
                stderr: "".into(),
            },
        ),
        (
            "/etc/containers/systemd/parent.volume".into(),
            RunCommandResult {
                success: true,
                stdout: "4e8dd3bfb2135d432cbbc00fb4da890319059df8f583fc8e324d73ba1e9d2b51 /etc/containers/systemd/parent.volume".into(),
                stderr: "".into(),
            },
        )
    ]));
    remote_check::remote_check(&connector, &config).await?;
    Ok(())
}

// Sorted from highest to lowest priority
// TODO: remote hash doesn't match local hash. This means that remote check should return a report
// TODO: unable to parse hash stdout (I.E. one string instead of two with space)
// TODO: remote command fails (!success)
// TODO: empty inventory
// TODO: multiple nodes
// TODO: local file read failure
// TODO: ssh connection failure
// TODO: ssh run_command Result::Err propagation
// TODO: invalid utf-8 quadlet path
// TODO: should the remote check also check the config hashes that were saved on the server and validate if those hashes
//       match the ones computed in the config
