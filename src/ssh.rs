use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to execute command on node: {0}")]
    OpenSsh(#[from] openssh::Error),
}

pub trait RemoteConnector {
    type Client: RemoteExecutor;
    async fn connect(&self, host: &str, user: &str) -> Result<Self::Client, Error>;
}

pub trait RemoteExecutor {
    async fn run_command(&self, command: &str, args: &[&str]) -> Result<RunCommandResult, Error>;
}

pub struct SshRemoteConnector;

impl RemoteConnector for SshRemoteConnector {
    type Client = SshClient;

    async fn connect(&self, host: &str, user: &str) -> Result<Self::Client, Error> {
        let session = openssh::Session::connect_mux(
            format!("{}@{}", user, host),
            openssh::KnownHosts::Strict,
        )
        .await?;
        Ok(SshClient { session })
    }
}

#[derive(Clone)]
pub struct RunCommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub struct SshClient {
    pub session: openssh::Session,
}

impl RemoteExecutor for SshClient {
    async fn run_command(&self, command: &str, args: &[&str]) -> Result<RunCommandResult, Error> {
        let result = self.session.command(command).args(args).output().await?;

        Ok(RunCommandResult {
            success: result.status.success(),
            stdout: String::from_utf8_lossy(&result.stdout).to_string(),
            stderr: String::from_utf8_lossy(&result.stderr).to_string(),
        })
    }
}
