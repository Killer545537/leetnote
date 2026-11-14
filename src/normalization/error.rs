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

    #[error("Transformation references unknown candidate: {0}")]
    UnknownCandidate(String),

    #[error("Candidate transformed more than once: {0}")]
    DuplicateTransformation(String),

    #[error("Empty LaTeX output for candidate: {original}")]
    EmptyLatex { original: String },

    #[error("LaTeX output must be wrapped in inline $...$ delimiters for {original}: {latex}")]
    InvalidLatexDelimiters { original: String, latex: String },

    #[error("Candidate '{original}' does not match source span {start}..{end}")]
    InvalidCandidateSpan {
        original: String,
        start: usize,
        end: usize,
    },
}

pub type Result<T> = std::result::Result<T, NormalizationError>;
