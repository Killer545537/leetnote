use thiserror::Error;

#[derive(Debug, Error)]
pub enum LeetcodeError {
    #[error("invalid Leetcode URL: {0}")]
    InvalidUrl(String),

    #[error("HTTP request failed")]
    Http(#[from] reqwest::Error),

    #[error("GraphQL request failed: {0}")]
    GraphQL(String),

    #[error("problem not found: {0}")]
    ProblemNotFound(String),

    #[error("{0} is a premium problem; LeetCode doesn't return its statement without a subscription")]
    PremiumOnly(String),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
}

pub type Result<T> = std::result::Result<T, LeetcodeError>;
