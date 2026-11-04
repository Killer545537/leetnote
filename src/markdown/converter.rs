use std::sync::LazyLock;

use crate::markdown::error::MarkdownError;

use super::error::Result;
use htmd::HtmlToMarkdown;
use regex::{Captures, Regex};

pub struct MarkdownConverter {
    converter: HtmlToMarkdown,
}

impl MarkdownConverter {
    pub fn new() -> Self {
        let converter = HtmlToMarkdown::builder()
            .skip_tags(vec!["img", "script", "style", "noscript"])
            .build();

        Self { converter }
    }

    pub fn convert(&self, html: &str) -> Result<String> {
        let html = Self::preprocess_html(html);
        let markdown = self
            .converter
            .convert(&html)
            .map_err(|err| MarkdownError::Conversion(err.to_string()))?;

        Ok(Self::normalize_markdown(&markdown))
    }

    fn preprocess_html(html: &str) -> String {
        static SUPERSCRIPT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"(?is)<sup\b[^>]*>(.*?)</sup>").expect("valid superscript regex")
        });

        SUPERSCRIPT_REGEX
            .replace_all(html, |captures: &Captures<'_>| {
                format!("^{{{}}}", captures[1].trim())
            })
            .into_owned()
    }

    fn normalize_markdown(markdown: &str) -> String {
        let markdown = markdown.replace('\u{00a0}', " ");

        let mut output = String::with_capacity(markdown.len());
        let mut consecutive_blank_lines = 0;

        for line in markdown.lines() {
            let line = line.trim_end();

            if line.trim().is_empty() {
                consecutive_blank_lines += 1;

                if consecutive_blank_lines <= 1 {
                    output.push('\n');
                }
            } else {
                consecutive_blank_lines = 0;
                output.push_str(line);
                output.push('\n');
            }
        }

        output.trim().to_owned()
    }
}

impl Default for MarkdownConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn converter() -> MarkdownConverter {
        MarkdownConverter::new()
    }

    #[test]
    fn converts_paragraphs() {
        let html = "<p>Hello world.</p><p>Second paragraph.</p>";

        let markdown = converter().convert(html).unwrap();

        assert!(markdown.contains("Hello world."));
        assert!(markdown.contains("Second paragraph."));
    }

    #[test]
    fn converts_inline_formatting() {
        let html = r#"
            <p>
                This is <strong>bold</strong> and
                <em>italic</em> with <code>inline code</code>.
            </p>
        "#;

        let markdown = converter().convert(html).unwrap();

        assert!(markdown.contains("**bold**"));
        assert!(markdown.contains("*italic*"));
        assert!(markdown.contains("`inline code`"));
    }

    #[test]
    fn converts_links() {
        let html = r#"
            <a href="https://leetcode.com/problems/two-sum/">
                Two Sum
            </a>
        "#;

        let markdown = converter().convert(html).unwrap();

        assert!(markdown.contains("[Two Sum](https://leetcode.com/problems/two-sum/)"));
    }

    #[test]
    fn removes_images() {
        let html = r#"
            <p>Before image.</p>
            <img src="https://example.com/image.png" alt="Some image">
            <p>After image.</p>
        "#;

        let markdown = converter().convert(html).unwrap();

        assert!(markdown.contains("Before image."));
        assert!(markdown.contains("After image."));
        assert!(!markdown.contains("example.com"));
        assert!(!markdown.contains("Some image"));
    }

    #[test]
    fn converts_lists() {
        let html = r#"
            <ul>
                <li>First item</li>
                <li>Second item</li>
            </ul>
        "#;

        let markdown = converter().convert(html).unwrap();

        assert!(markdown.contains("First item"));
        assert!(markdown.contains("Second item"));
    }

    #[test]
    fn converts_code_blocks() {
        let html = r#"
            <pre>
                <code>
Input: nums = [2,7,11,15], target = 9
Output: [0,1]
                </code>
            </pre>
        "#;

        let markdown = converter().convert(html).unwrap();

        assert!(markdown.contains("Input: nums = [2,7,11,15], target = 9"));
        assert!(markdown.contains("Output: [0,1]"));
    }

    #[test]
    fn handles_empty_input() {
        let markdown = converter().convert("").unwrap();

        assert!(markdown.is_empty());
    }

    #[test]
    fn preserves_superscripts() {
        let html = r#"
            <ul>
                <li><code>1 &lt;= g.length &lt;= 3 * 10<sup>4</sup></code></li>
                <li><code>0 &lt;= s.length &lt;= 3 * 10<sup>4</sup></code></li>
                <li><code>1 &lt;= g[i], s[j] &lt;= 2<sup>31</sup> - 1</code></li>
            </ul>
        "#;

        let markdown = converter().convert(html).unwrap();

        assert!(markdown.contains("10^{4}"));
        assert!(markdown.contains("2^{31}"));
    }
}
