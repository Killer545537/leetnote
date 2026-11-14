use crate::normalization::error::Result;

#[allow(async_fn_in_trait)]
pub trait StatementNormalizer {
    async fn normalize(&self, markdown: &str) -> Result<String>;
}
