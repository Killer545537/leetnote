use std::cell::RefCell;

use anyhow::{Result, bail};
use chrono::NaiveDate;
use leetnote::{
    Difficulty, ExpressionTransformation, LeetcodeClient, MarkdownConverter, NormalizationError,
    NoteWriter, ObsidianError, Problem, StatementNormalizer, apply_transformations,
    extract_candidates,
    pipeline::{
        ConvertStatement, FetchProblem, NormalizeStatement, Pipeline, RenderNote, Stage, WriteNote,
    },
};

struct Record<'a> {
    name: &'static str,
    events: &'a RefCell<Vec<&'static str>>,
    fail: bool,
}

impl Stage for Record<'_> {
    type Input = String;
    type Output = String;

    async fn run(&self, mut input: String) -> Result<String> {
        self.events.borrow_mut().push(self.name);
        if self.fail {
            bail!("stage failed: {}", self.name);
        }
        input.push_str(self.name);
        Ok(input)
    }
}

#[tokio::test]
async fn executes_stages_in_order_and_passes_outputs_forward() {
    let events = RefCell::new(Vec::new());
    let stage = |name| Record {
        name,
        events: &events,
        fail: false,
    };
    let pipeline = Pipeline::new(stage("first"))
        .then(stage("second"))
        .then(stage("third"));

    assert_eq!(
        pipeline.run(String::new()).await.unwrap(),
        "firstsecondthird"
    );
    assert_eq!(*events.borrow(), ["first", "second", "third"]);
}

#[tokio::test]
async fn stops_at_the_first_failed_stage() {
    let events = RefCell::new(Vec::new());
    let pipeline = Pipeline::new(Record {
        name: "first",
        events: &events,
        fail: true,
    })
    .then(Record {
        name: "second",
        events: &events,
        fail: false,
    });

    let error = pipeline.run(String::new()).await.unwrap_err();
    assert_eq!(error.to_string(), "stage failed: first");
    assert_eq!(*events.borrow(), ["first"]);
}

struct MockNormalizer {
    fail: bool,
}

impl StatementNormalizer for MockNormalizer {
    async fn normalize(&self, markdown: &str) -> Result<String, NormalizationError> {
        if self.fail {
            return Err(NormalizationError::UnknownCandidate("fabricated".into()));
        }
        apply_transformations(
            markdown,
            &extract_candidates(markdown),
            &[ExpressionTransformation {
                original: "i^{th}".into(),
                latex: "$i^{th}$".into(),
            }],
        )
    }
}

fn problem() -> Problem {
    Problem {
        question_frontend_id: "455".into(),
        title: "Assign Cookies".into(),
        title_slug: "assign-cookies".into(),
        difficulty: Difficulty::Easy,
        content: "<p>The <code>i^{th}</code> child uses <code>nums[i]</code>.</p>".into(),
        topic_tags: Vec::new(),
    }
}

#[tokio::test]
async fn converts_normalizes_renders_and_writes_a_note() {
    let directory = tempfile::tempdir().unwrap();
    let pipeline = Pipeline::new(ConvertStatement::new(MarkdownConverter::new()))
        .then(NormalizeStatement::new(MockNormalizer { fail: false }))
        .then(RenderNote::new(
            NaiveDate::from_ymd_opt(2026, 11, 14).unwrap(),
        ))
        .then(WriteNote::new(NoteWriter::new(directory.path())));

    let path = pipeline.run(problem()).await.unwrap();
    assert_eq!(path, directory.path().join("Assign Cookies.md"));
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("created: 2026-11-14"));
    assert!(content.contains("number: '455'"));
    assert!(content.contains("> **Assign Cookies**"));
    assert!(content.contains("> The $i^{th}$ child uses `nums[i]`."));
    assert!(content.contains("## Observations"));

    let error = pipeline.run(problem()).await.unwrap_err();
    assert!(matches!(
        error.downcast_ref::<ObsidianError>(),
        Some(ObsidianError::AlreadyExists(_))
    ));
    assert_eq!(std::fs::read_to_string(path).unwrap(), content);
}

#[tokio::test]
async fn normalization_failure_prevents_writing_and_preserves_the_error_source() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("notes");
    let pipeline = Pipeline::new(ConvertStatement::new(MarkdownConverter::new()))
        .then(NormalizeStatement::new(MockNormalizer { fail: true }))
        .then(RenderNote::new(
            NaiveDate::from_ymd_opt(2026, 11, 14).unwrap(),
        ))
        .then(WriteNote::new(NoteWriter::new(&output)));

    let error = pipeline.run(problem()).await.unwrap_err();
    assert_eq!(
        error.to_string(),
        "Failed to normalize mathematical expressions"
    );
    assert!(matches!(
        error.downcast_ref::<NormalizationError>(),
        Some(NormalizationError::UnknownCandidate(_))
    ));
    assert!(!output.exists());
}

#[tokio::test]
async fn fetch_stage_rejects_invalid_urls_before_network_access() {
    let pipeline = Pipeline::new(FetchProblem::new(LeetcodeClient::new().unwrap()));
    let error = pipeline
        .run("https://example.com/problems/two-sum/".into())
        .await
        .unwrap_err();

    assert_eq!(error.to_string(), "Invalid LeetCode problem URL");
}
