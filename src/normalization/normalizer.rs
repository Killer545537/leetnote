use crate::normalization::error::Result;

#[allow(async_fn_in_trait)]
pub trait StatementNormalizer {
    async fn normalize(&self, markdown: &str) -> Result<String>;
}

/// `None` leaves the statement as written (`--no-normalize`).
impl<N: StatementNormalizer> StatementNormalizer for Option<N> {
    async fn normalize(&self, markdown: &str) -> Result<String> {
        match self {
            Some(normalizer) => normalizer.normalize(markdown).await,
            None => Ok(markdown.to_owned()),
        }
    }
}
