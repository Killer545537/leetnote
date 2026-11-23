mod candidate;
mod error;
mod fm;
mod normalizer;
mod replacement;

pub use candidate::{Candidate, extract_candidates};
pub use error::NormalizationError;
pub use fm::{FmNormalizer, SCHEMA};
pub use normalizer::StatementNormalizer;
pub use replacement::{
    ExpressionTransformation, NormalizationResponse, apply_transformations, validate_transformations,
};
