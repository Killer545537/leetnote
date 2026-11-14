use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NormalizationError {
    #[error("Foundation Models process failed: {0}")]
    Io(#[from] io::Error),

    #[error("Foundation Models exited unsuccessfully ({status:?}): {stderr}")]
    ProcessFailed {
        status: std::process::ExitStatus,
        stderr: String,
    },

    #[error("Invalid response from Foundation Models")]
    InvalidResponse(#[from] serde_json::Error),

    #[error("Foundation Models returned empty Markdown document")]
    EmptyMarkdown,
}

pub type Result<T> = std::result::Result<T, NormalizationError>;
