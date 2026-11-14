use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use leetnote::{
    ExpressionTransformation, FmNormalizer, NormalizationError, NormalizationResponse,
    StatementNormalizer, apply_transformations, extract_candidates,
};
use tempfile::TempDir;

static MOCK_FM_DIR: LazyLock<TempDir> = LazyLock::new(|| {
    let dir = tempfile::tempdir().expect("create temp dir for mock fm");
    let script_path = dir.path().join("mock-fm");
    let script = r#"#!/bin/sh
set -eu
if [ "$1" != "respond" ] || [ "$2" != "--instructions" ] || [ "$4" != "--schema" ]; then
    echo "unexpected arguments: $*" >&2
    exit 2
fi
schema_file="$5"
prompt="$6"
printf '%s' "$prompt" > "${schema_file}.prompt"
if [ -f "${schema_file}.err" ]; then
    cat "${schema_file}.err" >&2
    exit 1
fi
cat "$schema_file"
"#;
    fs::write(&script_path, script).expect("write mock fm script");
    let mut perms = fs::metadata(&script_path)
        .expect("stat mock fm")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).expect("chmod mock fm");
    dir
});

fn mock_fm_executable() -> PathBuf {
    MOCK_FM_DIR.path().join("mock-fm")
}

fn setup_mock_normalizer(dir: &TempDir, response_json: &str) -> (FmNormalizer, PathBuf) {
    let response_path = dir.path().join("response.json");
    fs::write(&response_path, response_json).expect("write mocked response json");
    let normalizer = FmNormalizer::new(&response_path).with_executable(mock_fm_executable());
    let prompt_path = PathBuf::from(format!("{}.prompt", response_path.display()));
    (normalizer, prompt_path)
}

fn read_recorded_prompt(prompt_path: &Path) -> Option<String> {
    fs::read_to_string(prompt_path).ok()
}

#[test]
fn schema_file_enforces_structured_expressions_contract() {
    let schema_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas/statement-normalization.json");
    let content = fs::read_to_string(schema_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["required"], serde_json::json!(["expressions"]));

    let items = &schema["properties"]["expressions"]["items"];
    assert_eq!(items["type"], "object");
    assert_eq!(items["additionalProperties"], false);
    assert_eq!(items["required"], serde_json::json!(["original", "latex"]));
    assert_eq!(items["properties"]["original"]["type"], "string");
    assert_eq!(items["properties"]["latex"]["type"], "string");
}

#[tokio::test]
async fn normalizes_mathematical_candidates() {
    let dir = tempfile::tempdir().unwrap();
    let response = serde_json::json!({
        "expressions": [
            {
                "original": "i^{th}",
                "latex": "$i^{th}$"
            },
            {
                "original": "j^{th}",
                "latex": "$j^{th}$"
            },
            {
                "original": "2 * 10^{5}",
                "latex": r"$2 \times 10^{5}$"
            },
            {
                "original": "1 <= n, m <= 2 * 10^{5}",
                "latex": r"$1 \leq n,m \leq 2 \times 10^{5}$"
            }
        ]
    });

    let (normalizer, prompt_path) = setup_mock_normalizer(&dir, &response.to_string());

    let markdown = "\
The `i^{th}` server processes the `j^{th}` task in at most `2 * 10^{5}` steps.

Constraints:
* `1 <= n, m <= 2 * 10^{5}`";

    let normalized = normalizer.normalize(markdown).await.unwrap();

    assert_eq!(
        normalized,
        "\
The $i^{th}$ server processes the $j^{th}$ task in at most $2 \\times 10^{5}$ steps.

Constraints:
* $1 \\leq n,m \\leq 2 \\times 10^{5}$"
    );

    let prompt = read_recorded_prompt(&prompt_path).unwrap();
    assert!(prompt.contains("i^{th}"));
    assert!(prompt.contains("j^{th}"));
    assert!(prompt.contains("2 * 10^{5}"));
    assert!(prompt.contains("1 <= n, m <= 2 * 10^{5}"));
}

#[tokio::test]
async fn leaves_programming_expressions_unchanged_and_unextracted() {
    let dir = tempfile::tempdir().unwrap();
    let (normalizer, prompt_path) = setup_mock_normalizer(&dir, r#"{"expressions": []}"#);

    let markdown = "\
Given `servers[i]`, `tasks[j]`, `nums.length`, and `ans[j]`:
Example input: `nums = [1, 2, 3]`.";

    let candidates = extract_candidates(markdown);
    assert!(candidates.is_empty());

    let normalized = normalizer.normalize(markdown).await.unwrap();
    assert_eq!(normalized, markdown);
    // Process is not even invoked when no candidates exist
    assert!(read_recorded_prompt(&prompt_path).is_none());
}

#[tokio::test]
async fn preserves_fenced_code_blocks_links_headings_lists_and_prose_byte_for_byte() {
    let dir = tempfile::tempdir().unwrap();
    let response = serde_json::json!({
        "expressions": [
            {
                "original": "i^{th}",
                "latex": "$i^{th}$"
            },
            {
                "original": "2 * 10^{5}",
                "latex": r"$2 \times 10^{5}$"
            }
        ]
    });

    let (normalizer, prompt_path) = setup_mock_normalizer(&dir, &response.to_string());

    let markdown = "\
# Problem Heading

Assign the `i^{th}` task using `servers[i]` and `tasks[j]`, checking `nums.length` and `ans[j]`.

```python
# Fenced code block must stay byte-for-byte identical
for i in range(2 * 10^{5}):
    # i^{th} item when 1 <= n, m <= 2 * 10^{5}
    nums = [1, 2, 3]
    ans[j] = servers[i]
```

## Notes & Links

* First item with `nums = [1, 2, 3]`
* Second item with `2 * 10^{5}` limit
* Reference: [Problem 2 * 10^{5}](https://leetcode.com/problems/assign-cookies/?envId=2026-11-07)
";

    let expected = "\
# Problem Heading

Assign the $i^{th}$ task using `servers[i]` and `tasks[j]`, checking `nums.length` and `ans[j]`.

```python
# Fenced code block must stay byte-for-byte identical
for i in range(2 * 10^{5}):
    # i^{th} item when 1 <= n, m <= 2 * 10^{5}
    nums = [1, 2, 3]
    ans[j] = servers[i]
```

## Notes & Links

* First item with `nums = [1, 2, 3]`
* Second item with $2 \\times 10^{5}$ limit
* Reference: [Problem 2 * 10^{5}](https://leetcode.com/problems/assign-cookies/?envId=2026-11-07)
";

    let normalized = normalizer.normalize(markdown).await.unwrap();
    assert_eq!(normalized, expected);

    // Ensure fenced code and programming identifiers were never sent to the model
    let prompt = read_recorded_prompt(&prompt_path).unwrap();
    assert!(!prompt.contains("for i in range"));
    assert!(!prompt.contains("servers[i]"));
    assert!(!prompt.contains("tasks[j]"));
    assert!(!prompt.contains("nums.length"));
    assert!(!prompt.contains("ans[j]"));
    assert!(!prompt.contains("nums = [1, 2, 3]"));
    assert!(!prompt.contains("https://leetcode.com"));
}

#[tokio::test]
async fn handles_repeated_expressions_across_contexts_correctly() {
    let dir = tempfile::tempdir().unwrap();
    let response = serde_json::json!({
        "expressions": [
            {
                "original": "i^{th}",
                "latex": "$i^{th}$"
            }
        ]
    });

    let (normalizer, _) = setup_mock_normalizer(&dir, &response.to_string());

    let markdown = "\
First `i^{th}` occurrence and second `i^{th}` occurrence, plus plain i^{th} occurrence.

```text
Code fence i^{th} and `i^{th}` must not change.
```

Link [i^{th}](https://leetcode.com/problems/i^{th}/) must not change.";

    let expected = "\
First $i^{th}$ occurrence and second $i^{th}$ occurrence, plus plain $i^{th}$ occurrence.

```text
Code fence i^{th} and `i^{th}` must not change.
```

Link [i^{th}](https://leetcode.com/problems/i^{th}/) must not change.";

    let normalized = normalizer.normalize(markdown).await.unwrap();
    assert_eq!(normalized, expected);
}

#[tokio::test]
async fn empty_transformation_response_leaves_document_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let (normalizer, prompt_path) = setup_mock_normalizer(&dir, r#"{"expressions": []}"#);

    let markdown = "The `i^{th}` server has `1 <= n <= 2 * 10^{5}` capacity.";
    let normalized = normalizer.normalize(markdown).await.unwrap();

    assert_eq!(normalized, markdown);
    assert!(read_recorded_prompt(&prompt_path).is_some());
}

#[tokio::test]
async fn rejects_fabricated_or_invalid_model_transformations() {
    let markdown = "The `i^{th}` server handles `2 * 10^{5}` tasks.";

    // 1. Fabricated candidate that was not extracted from the document
    {
        let dir = tempfile::tempdir().unwrap();
        let response = serde_json::json!({
            "expressions": [
                {
                    "original": "The `i^{th}` server",
                    "latex": "$hacked$"
                }
            ]
        });
        let (normalizer, _) = setup_mock_normalizer(&dir, &response.to_string());
        let err = normalizer.normalize(markdown).await.unwrap_err();
        assert!(matches!(err, NormalizationError::UnknownCandidate(_)));
    }

    // 2. Duplicate transformation of the same candidate
    {
        let dir = tempfile::tempdir().unwrap();
        let response = serde_json::json!({
            "expressions": [
                {
                    "original": "i^{th}",
                    "latex": "$i^{th}$"
                },
                {
                    "original": "i^{th}",
                    "latex": "$i^{th}$"
                }
            ]
        });
        let (normalizer, _) = setup_mock_normalizer(&dir, &response.to_string());
        let err = normalizer.normalize(markdown).await.unwrap_err();
        assert!(matches!(
            err,
            NormalizationError::DuplicateTransformation(_)
        ));
    }

    // 3. Missing inline math delimiters
    {
        let dir = tempfile::tempdir().unwrap();
        let response = serde_json::json!({
            "expressions": [
                {
                    "original": "i^{th}",
                    "latex": "i^{th}"
                }
            ]
        });
        let (normalizer, _) = setup_mock_normalizer(&dir, &response.to_string());
        let err = normalizer.normalize(markdown).await.unwrap_err();
        assert!(matches!(
            err,
            NormalizationError::InvalidLatexDelimiters { .. }
        ));
    }

    // 4. Empty LaTeX content
    {
        let dir = tempfile::tempdir().unwrap();
        let response = serde_json::json!({
            "expressions": [
                {
                    "original": "i^{th}",
                    "latex": "$$"
                }
            ]
        });
        let (normalizer, _) = setup_mock_normalizer(&dir, &response.to_string());
        let err = normalizer.normalize(markdown).await.unwrap_err();
        assert!(matches!(err, NormalizationError::EmptyLatex { .. }));
    }

    // 5. Additional unexpected properties in JSON response
    {
        let dir = tempfile::tempdir().unwrap();
        let response = serde_json::json!({
            "expressions": [
                {
                    "original": "i^{th}",
                    "latex": "$i^{th}$",
                    "extra": "disallowed"
                }
            ]
        });
        let (normalizer, _) = setup_mock_normalizer(&dir, &response.to_string());
        let err = normalizer.normalize(markdown).await.unwrap_err();
        assert!(matches!(err, NormalizationError::InvalidResponse(_)));
    }
}

#[test]
fn deterministic_replacement_preserves_untransformed_candidates() {
    let markdown = "Compare `i^{th}` and `1 <= n <= 2^{31} - 1` in prose.";
    let candidates = extract_candidates(markdown);
    assert_eq!(candidates.len(), 2);

    // Model converts `i^{th}` but excludes `1 <= n <= 2^{31} - 1`
    let response = NormalizationResponse {
        expressions: vec![ExpressionTransformation {
            original: "i^{th}".into(),
            latex: "$i^{th}$".into(),
        }],
    };

    let result = apply_transformations(markdown, &candidates, &response.expressions).unwrap();
    assert_eq!(
        result,
        "Compare $i^{th}$ and `1 <= n <= 2^{31} - 1` in prose."
    );
}
