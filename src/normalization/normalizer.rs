use crate::normalization::error::Result;

pub trait StatementNormalizer {
    async fn normalize(&self, markdown: &str) -> Result<String>;
}
