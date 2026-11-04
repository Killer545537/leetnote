use thiserror::Error;

#[derive(Debug, Error)]
pub enum MarkdownError {
    #[error("HTML-to-MD conversion failed: {0}")]
    Conversion(String),
}

pub type Result<T> = std::result::Result<T, MarkdownError>;
