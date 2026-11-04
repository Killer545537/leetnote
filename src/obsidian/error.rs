use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObsidianError {
    #[error("Invalid YAML value: {0}")]
    InvalidYaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, ObsidianError>;
