use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObsidianError {
    #[error("Failed to access {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Note already exists: {0}")]
    AlreadyExists(PathBuf),

    #[error("Invalid YAML value: {0}")]
    InvalidYaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, ObsidianError>;
