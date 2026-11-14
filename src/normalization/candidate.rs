use std::{ops::Range, sync::LazyLock};

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub original: String,
    pub span: Range<usize>,
}

pub fn extract_candidates(markdown: &str) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut active_fence: Option<(u8, usize)> = None;
    let mut offset = 0;

    for line in markdown.split_inclusive('\n') {
        let line_start = offset;
        offset += line.len();

        let line_without_newline = line.strip_suffix('\n').unwrap_or(line);
        let line_without_cr = line_without_newline
            .strip_suffix('\r')
            .unwrap_or(line_without_newline);

        if let Some((fence_char, fence_len)) = active_fence {
            if is_closing_fence(line_without_cr, fence_char, fence_len) {
                active_fence = None;
            }
            continue;
        }

        if let Some(fence) = detect_opening_fence(line_without_cr) {
            active_fence = Some(fence);
            continue;
        }

        extract_line_candidates(line_without_cr, line_start, &mut candidates);
    }

    candidates
}

fn strip_blockquote_and_indent(line: &str) -> &str {
    let mut rest = line.trim_start_matches([' ', '\t']);
    while let Some(after_gt) = rest.strip_prefix('>') {
        rest = after_gt.trim_start_matches([' ', '\t']);
    }
    rest
}

fn detect_opening_fence(line: &str) -> Option<(u8, usize)> {
    let trimmed = strip_blockquote_and_indent(line);
    let bytes = trimmed.as_bytes();
    let first = *bytes.first()?;

    if first != b'`' && first != b'~' {
        return None;
    }

    let count = bytes.iter().take_while(|&&b| b == first).count();
    if count < 3 {
        return None;
    }

    let info_string = &trimmed[count..];
    if first == b'`' && info_string.contains('`') {
        return None;
    }

    Some((first, count))
}

fn is_closing_fence(line: &str, fence_char: u8, min_len: usize) -> bool {
    let trimmed = strip_blockquote_and_indent(line);
    let bytes = trimmed.as_bytes();
    let count = bytes.iter().take_while(|&&b| b == fence_char).count();

    count >= min_len && trimmed[count..].trim().is_empty()
}

fn skip_line_prefix(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut idx = 0;

    // Skip leading whitespace and blockquote `>` markers
    loop {
        while idx < bytes.len() && matches!(bytes[idx], b' ' | b'\t') {
            idx += 1;
        }
        if idx < bytes.len() && bytes[idx] == b'>' {
            idx += 1;
            continue;
        }
        break;
    }

    // Skip ATX heading markers (`#`, `##`, etc.) followed by space
    let heading_start = idx;
    while idx < bytes.len() && bytes[idx] == b'#' {
        idx += 1;
    }
    if idx > heading_start {
        if idx < bytes.len() && matches!(bytes[idx], b' ' | b'\t') {
            while idx < bytes.len() && matches!(bytes[idx], b' ' | b'\t') {
                idx += 1;
            }
            return idx;
        }
        idx = heading_start;
    }

    // Skip unordered list markers (`* `, `- `, `+ `)
    if idx + 1 < bytes.len()
        && matches!(bytes[idx], b'*' | b'-' | b'+')
        && matches!(bytes[idx + 1], b' ' | b'\t')
    {
        idx += 1;
        while idx < bytes.len() && matches!(bytes[idx], b' ' | b'\t') {
            idx += 1;
        }
        return idx;
    }

    // Skip ordered list markers (`1. `, `2) `)
    let digits_start = idx;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        idx += 1;
    }
    if idx > digits_start
        && idx + 1 < bytes.len()
        && matches!(bytes[idx], b'.' | b')')
        && matches!(bytes[idx + 1], b' ' | b'\t')
    {
        idx += 1;
        while idx < bytes.len() && matches!(bytes[idx], b' ' | b'\t') {
            idx += 1;
        }
        return idx;
    }
    idx = digits_start;

    idx
}

fn extract_line_candidates(line: &str, line_offset: usize, candidates: &mut Vec<Candidate>) {
    let start_idx = skip_line_prefix(line);
    let bytes = line.as_bytes();
    let mut idx = start_idx;
    let mut plain_start = start_idx;

    while idx < bytes.len() {
        match bytes[idx] {
            b'`' => {
                let tick_count = bytes[idx..].iter().take_while(|&&b| b == b'`').count();
                let content_start = idx + tick_count;

                if let Some(close_rel) = find_closing_backticks(&bytes[content_start..], tick_count)
                {
                    extract_plain_text_candidates(
                        &line[plain_start..idx],
                        line_offset + plain_start,
                        candidates,
                    );

                    let close_start = content_start + close_rel;
                    let close_end = close_start + tick_count;
                    let inner = strip_zero_width(line[content_start..close_start].trim());

                    if is_inline_code_math_candidate(&inner) {
                        candidates.push(Candidate {
                            original: inner,
                            span: (line_offset + idx)..(line_offset + close_end),
                        });
                    }

                    idx = close_end;
                    plain_start = close_end;
                } else {
                    idx += tick_count;
                }
            }
            b'!' if idx + 1 < bytes.len() && bytes[idx + 1] == b'[' => {
                if let Some(link_end) = find_markdown_link_end(line, idx + 1) {
                    extract_plain_text_candidates(
                        &line[plain_start..idx],
                        line_offset + plain_start,
                        candidates,
                    );
                    idx = link_end;
                    plain_start = link_end;
                } else {
                    idx += 1;
                }
            }
            b'[' => {
                if let Some(link_end) = find_markdown_link_end(line, idx) {
                    extract_plain_text_candidates(
                        &line[plain_start..idx],
                        line_offset + plain_start,
                        candidates,
                    );
                    idx = link_end;
                    plain_start = link_end;
                } else {
                    idx += 1;
                }
            }
            b'<' if line[idx..].starts_with("<https://") || line[idx..].starts_with("<http://") => {
                if let Some(rel_end) = line[idx..].find('>') {
                    extract_plain_text_candidates(
                        &line[plain_start..idx],
                        line_offset + plain_start,
                        candidates,
                    );
                    idx += rel_end + 1;
                    plain_start = idx;
                } else {
                    idx += 1;
                }
            }
            b'$' => {
                let delim_len = if idx + 1 < bytes.len() && bytes[idx + 1] == b'$' {
                    2
                } else {
                    1
                };
                let search_start = idx + delim_len;
                let delim = &line[idx..search_start];

                if let Some(rel_close) = line[search_start..].find(delim) {
                    extract_plain_text_candidates(
                        &line[plain_start..idx],
                        line_offset + plain_start,
                        candidates,
                    );
                    idx = search_start + rel_close + delim_len;
                    plain_start = idx;
                } else {
                    idx += delim_len;
                }
            }
            _ => {
                idx += 1;
            }
        }
    }

    if plain_start < bytes.len() {
        extract_plain_text_candidates(&line[plain_start..], line_offset + plain_start, candidates);
    }
}

fn find_closing_backticks(bytes: &[u8], tick_count: usize) -> Option<usize> {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            let run = bytes[i..].iter().take_while(|&&b| b == b'`').count();
            if run == tick_count {
                return Some(i);
            }
            i += run;
        } else {
            i += 1;
        }
    }
    None
}

fn find_markdown_link_end(line: &str, bracket_start: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    if bytes.get(bracket_start) != Some(&b'[') {
        return None;
    }

    let mut depth = 0usize;
    let mut i = bracket_start;
    let mut close_bracket = None;

    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'`' => {
                return None;
            }
            b'[' => depth += 1,
            b']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    close_bracket = Some(i);
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let close_bracket = close_bracket?;
    let paren_start = close_bracket + 1;
    if bytes.get(paren_start) != Some(&b'(') {
        return None;
    }

    let mut paren_depth = 0usize;
    let mut j = paren_start;
    while j < bytes.len() {
        match bytes[j] {
            b'\\' => {
                j += 2;
                continue;
            }
            b'(' => paren_depth += 1,
            b')' => {
                paren_depth = paren_depth.saturating_sub(1);
                if paren_depth == 0 {
                    return Some(j + 1);
                }
            }
            _ => {}
        }
        j += 1;
    }

    None
}

fn is_inline_code_math_candidate(inner: &str) -> bool {
    if inner.is_empty() || inner.contains(['\n', '\r']) {
        return false;
    }

    if has_programming_syntax(inner) {
        return false;
    }

    static SUPERSCRIPT_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"[A-Za-z0-9)}\s]\^(?:\{[^}]+\}|[0-9A-Za-z]+)").expect("valid superscript regex")
    });

    static INEQUALITY_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"[A-Za-z0-9)}]\s*(?:<=|>=|!=)\s*[-+A-Za-z0-9({]|[A-Za-z0-9)}]\s+[<>]\s+[-+A-Za-z0-9({]",
        )
        .expect("valid inequality regex")
    });

    static MULTIPLICATION_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"[A-Za-z0-9)}]\s*\*\s*[A-Za-z0-9({]").expect("valid multiplication regex")
    });

    SUPERSCRIPT_RE.is_match(inner)
        || INEQUALITY_RE.is_match(inner)
        || MULTIPLICATION_RE.is_match(inner)
}

fn has_programming_syntax(input: &str) -> bool {
    // Reject array indexing/literals (`nums[i]`, `servers[i]`, `nums = [1, 2, 3]`)
    // and statements ending with `;`
    if input.contains(['[', ']', ';']) || has_standalone_assignment(input) {
        return true;
    }

    // Reject member/property access (`nums.length`, `servers.length`)
    static MEMBER_ACCESS_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"[A-Za-z_][A-Za-z0-9_]*\.[A-Za-z_][A-Za-z0-9_]*")
            .expect("valid member access regex")
    });

    MEMBER_ACCESS_RE.is_match(input)
}

fn has_standalone_assignment(input: &str) -> bool {
    let bytes = input.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b'=' {
            let prev = i.checked_sub(1).and_then(|p| bytes.get(p)).copied();
            let next = bytes.get(i + 1).copied();

            let part_of_comparison = matches!(prev, Some(b'<' | b'>' | b'!' | b'='))
                || matches!(next, Some(b'=' | b'>'));

            if !part_of_comparison {
                return true;
            }
        }
    }
    false
}

fn extract_plain_text_candidates(
    segment: &str,
    base_offset: usize,
    candidates: &mut Vec<Candidate>,
) {
    if segment.trim().is_empty() {
        return;
    }

    // Plain-text mathematical patterns ordered from most composite to simplest:
    // 1. Inequalities and chained ranges (e.g. `1 <= n, m <= 2 * 10^{5}`, `1 <= n`)
    // 2. Arithmetic expressions involving superscripts or multiplication (e.g. `2 * 10^{5}`, `2^{31} - 1`)
    // 3. Standalone superscript notation (e.g. `i^{th}`, `j^{th}`, `10^{5}`)
    static PLAIN_MATH_RE: LazyLock<Regex> = LazyLock::new(|| {
        let sup = r"\^(?:\{[^}\n]+\}|[0-9]+)";
        let atom = format!(r"(?:[0-9]+(?:{sup})?|[A-Za-z0-9]+{sup}|\b[A-Za-z]\b)");
        let term = format!(r"-?\s*{atom}(?:\s*[*+\-/]\s*{atom})*");
        let ineq = r"(?:<=|>=|!=|<|>)";

        let inequality_expr =
            format!(r"{term}(?:\s*{ineq}\s*(?:{term}(?:\s*,\s*{term})+\s*{ineq}\s*)*{term})+");
        let sup_atom = format!(r"[A-Za-z0-9]+{sup}");
        let arithmetic_with_sup = format!(
            r"(?:{atom}\s*[*+\-/]\s*)*{sup_atom}(?:\s*[*+\-/]\s*{atom})+|(?:{atom}\s*[*+\-/]\s*)+{sup_atom}"
        );
        let multiplication_expr = format!(r"[0-9]+\s*\*\s*{atom}");

        let pattern =
            format!(r"{inequality_expr}|{arithmetic_with_sup}|{multiplication_expr}|{sup_atom}");
        Regex::new(&pattern).expect("valid plain-text math regex")
    });

    for matched in PLAIN_MATH_RE.find_iter(segment) {
        let raw = matched.as_str();
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        let leading_ws = raw.len() - raw.trim_start().len();
        let start = base_offset + matched.start() + leading_ws;
        let end = start + trimmed.len();
        let clean_original = strip_zero_width(trimmed);

        candidates.push(Candidate {
            original: clean_original,
            span: start..end,
        });
    }
}

fn strip_zero_width(input: &str) -> String {
    input
        .chars()
        .filter(|&ch| !matches!(ch, '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{feff}'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_superscript_candidates() {
        let markdown = "Assign the `i^{th}` task to the `j^{th}` server.";
        let candidates = extract_candidates(markdown);

        assert_eq!(
            candidates,
            vec![
                Candidate {
                    original: "i^{th}".into(),
                    span: 11..19,
                },
                Candidate {
                    original: "j^{th}".into(),
                    span: 32..40,
                },
            ]
        );
        assert_eq!(&markdown[candidates[0].span.clone()], "`i^{th}`");
        assert_eq!(&markdown[candidates[1].span.clone()], "`j^{th}`");
    }

    #[test]
    fn extracts_plain_text_superscripts_and_constraints() {
        let markdown =
            "The i^{th} server and j^{th} task with 2 * 10^{5} limit:\n- 1 <= n, m <= 2 * 10^{5}";
        let candidates = extract_candidates(markdown);

        let originals: Vec<&str> = candidates.iter().map(|c| c.original.as_str()).collect();
        assert_eq!(
            originals,
            vec!["i^{th}", "j^{th}", "2 * 10^{5}", "1 <= n, m <= 2 * 10^{5}"]
        );
    }

    #[test]
    fn ignores_programming_expressions_and_indexed_constraints() {
        let markdown = "\
Use `servers[i]`, `tasks[j]`, `nums.length`, `ans[j]`, and `nums = [1, 2, 3]`.
Constraints:
* `1 <= nums.length <= 100`
* `1 <= l <= r <= nums.length`
* `-1000 <= nums[i] <= 1000`";
        let candidates = extract_candidates(markdown);

        assert!(candidates.is_empty());
    }

    #[test]
    fn ignores_fenced_code_blocks() {
        let markdown = "\
Outside `i^{th}` candidate.

```rust
let x = 2 * 10^{5};
// 1 <= n, m <= 2 * 10^{5}
// i^{th}
```

~~~python
assert 1 <= n <= 2 * 10^{5}
~~~
";
        let candidates = extract_candidates(markdown);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].original, "i^{th}");
    }

    #[test]
    fn ignores_markdown_links() {
        let markdown =
            "See [2410: Problem 10^{5}](https://leetcode.com/problems/test/) and `1 <= n`.";
        let candidates = extract_candidates(markdown);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].original, "1 <= n");
    }
}
