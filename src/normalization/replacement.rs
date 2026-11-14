use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::{
    candidate::Candidate,
    error::{NormalizationError, Result},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpressionTransformation {
    pub original: String,
    pub latex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizationResponse {
    pub expressions: Vec<ExpressionTransformation>,
}

pub fn validate_transformations(
    markdown: &str,
    candidates: &[Candidate],
    transformations: &[ExpressionTransformation],
) -> Result<()> {
    let mut prev_end = 0usize;

    for candidate in candidates {
        let start = candidate.span.start;
        let end = candidate.span.end;

        if start >= end
            || end > markdown.len()
            || start < prev_end
            || !markdown.is_char_boundary(start)
            || !markdown.is_char_boundary(end)
        {
            return Err(NormalizationError::InvalidCandidateSpan {
                original: candidate.original.clone(),
                start,
                end,
            });
        }

        let slice = &markdown[start..end];
        if !matches_source_slice(slice, &candidate.original) {
            return Err(NormalizationError::InvalidCandidateSpan {
                original: candidate.original.clone(),
                start,
                end,
            });
        }

        prev_end = end;
    }

    let valid_originals: HashSet<&str> = candidates
        .iter()
        .map(|candidate| candidate.original.as_str())
        .collect();

    let mut seen_originals: HashSet<&str> = HashSet::with_capacity(transformations.len());

    for transformation in transformations {
        if !valid_originals.contains(transformation.original.as_str()) {
            return Err(NormalizationError::UnknownCandidate(
                transformation.original.clone(),
            ));
        }

        if !seen_originals.insert(transformation.original.as_str()) {
            return Err(NormalizationError::DuplicateTransformation(
                transformation.original.clone(),
            ));
        }

        validate_latex(&transformation.original, &transformation.latex)?;
    }

    Ok(())
}

pub fn apply_transformations(
    markdown: &str,
    candidates: &[Candidate],
    transformations: &[ExpressionTransformation],
) -> Result<String> {
    validate_transformations(markdown, candidates, transformations)?;

    if transformations.is_empty() || candidates.is_empty() {
        return Ok(markdown.to_owned());
    }

    let lookup: HashMap<&str, &str> = transformations
        .iter()
        .map(|item| (item.original.as_str(), item.latex.as_str()))
        .collect();

    let mut output = String::with_capacity(markdown.len());
    let mut cursor = 0usize;

    for candidate in candidates {
        if let Some(&latex) = lookup.get(candidate.original.as_str()) {
            output.push_str(&markdown[cursor..candidate.span.start]);
            output.push_str(latex);
            cursor = candidate.span.end;
        }
    }

    output.push_str(&markdown[cursor..]);

    Ok(output)
}

fn matches_source_slice(slice: &str, expected_original: &str) -> bool {
    if strip_zero_width(slice) == expected_original {
        return true;
    }

    let bytes = slice.as_bytes();
    let tick_count = bytes.iter().take_while(|&&b| b == b'`').count();
    if tick_count == 0 || bytes.len() < tick_count * 2 {
        return false;
    }

    let trailing_ticks = bytes.iter().rev().take_while(|&&b| b == b'`').count();
    if trailing_ticks != tick_count {
        return false;
    }

    let inner = slice[tick_count..slice.len() - tick_count].trim();
    strip_zero_width(inner) == expected_original
}

fn strip_zero_width(input: &str) -> String {
    input
        .chars()
        .filter(|&ch| !matches!(ch, '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{feff}'))
        .collect()
}

fn validate_latex(original: &str, latex: &str) -> Result<()> {
    if latex.trim().is_empty() || latex.trim() == "$$" {
        return Err(NormalizationError::EmptyLatex {
            original: original.to_owned(),
        });
    }

    if !latex.starts_with('$')
        || !latex.ends_with('$')
        || latex.starts_with("$$")
        || latex.ends_with("$$")
    {
        return Err(NormalizationError::InvalidLatexDelimiters {
            original: original.to_owned(),
            latex: latex.to_owned(),
        });
    }

    let inner = &latex[1..latex.len() - 1];
    if inner.trim().is_empty() {
        return Err(NormalizationError::EmptyLatex {
            original: original.to_owned(),
        });
    }

    if inner.contains(['$', '\n', '\r']) {
        return Err(NormalizationError::InvalidLatexDelimiters {
            original: original.to_owned(),
            latex: latex.to_owned(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalization::candidate::extract_candidates;

    #[test]
    fn applies_valid_replacements_by_span() {
        let markdown = "The `i^{th}` server handles `2 * 10^{5}` tasks.";
        let candidates = extract_candidates(markdown);
        let transformations = vec![
            ExpressionTransformation {
                original: "i^{th}".into(),
                latex: "$i^{th}$".into(),
            },
            ExpressionTransformation {
                original: "2 * 10^{5}".into(),
                latex: r"$2 \times 10^{5}$".into(),
            },
        ];

        let normalized = apply_transformations(markdown, &candidates, &transformations).unwrap();
        assert_eq!(
            normalized,
            r"The $i^{th}$ server handles $2 \times 10^{5}$ tasks."
        );
    }

    #[test]
    fn rejects_unknown_candidate() {
        let markdown = "The `i^{th}` server.";
        let candidates = extract_candidates(markdown);
        let transformations = vec![ExpressionTransformation {
            original: "j^{th}".into(),
            latex: "$j^{th}$".into(),
        }];

        let err = apply_transformations(markdown, &candidates, &transformations).unwrap_err();
        assert!(matches!(err, NormalizationError::UnknownCandidate(ref s) if s == "j^{th}"));
    }

    #[test]
    fn rejects_duplicate_transformation() {
        let markdown = "The `i^{th}` server.";
        let candidates = extract_candidates(markdown);
        let transformations = vec![
            ExpressionTransformation {
                original: "i^{th}".into(),
                latex: "$i^{th}$".into(),
            },
            ExpressionTransformation {
                original: "i^{th}".into(),
                latex: "$i^{th}$".into(),
            },
        ];

        let err = apply_transformations(markdown, &candidates, &transformations).unwrap_err();
        assert!(matches!(err, NormalizationError::DuplicateTransformation(ref s) if s == "i^{th}"));
    }

    #[test]
    fn rejects_empty_latex() {
        let markdown = "The `i^{th}` server.";
        let candidates = extract_candidates(markdown);

        for empty in ["", "   ", "$$", "$   $"] {
            let transformations = vec![ExpressionTransformation {
                original: "i^{th}".into(),
                latex: empty.into(),
            }];
            let err = apply_transformations(markdown, &candidates, &transformations).unwrap_err();
            assert!(matches!(err, NormalizationError::EmptyLatex { .. }));
        }
    }

    #[test]
    fn rejects_invalid_latex_delimiters() {
        let markdown = "The `i^{th}` server.";
        let candidates = extract_candidates(markdown);

        for invalid in [
            "i^{th}",
            "$$i^{th}$$",
            "$i^{th}",
            "i^{th}$",
            "$i^{th}$ injected $x$",
            "$i^{th}\n$",
        ] {
            let transformations = vec![ExpressionTransformation {
                original: "i^{th}".into(),
                latex: invalid.into(),
            }];
            let err = apply_transformations(markdown, &candidates, &transformations).unwrap_err();
            assert!(matches!(
                err,
                NormalizationError::InvalidLatexDelimiters { .. }
            ));
        }
    }

    #[test]
    fn rejects_mismatched_candidate_span() {
        let markdown = "The `i^{th}` server.";
        let bogus_candidates = vec![Candidate {
            original: "i^{th}".into(),
            span: 0..3,
        }];
        let transformations = vec![ExpressionTransformation {
            original: "i^{th}".into(),
            latex: "$i^{th}$".into(),
        }];

        let err = apply_transformations(markdown, &bogus_candidates, &transformations).unwrap_err();
        assert!(matches!(
            err,
            NormalizationError::InvalidCandidateSpan { .. }
        ));
    }
}
