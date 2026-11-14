use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::NaiveDate;

use crate::{
    LeetcodeClient, MarkdownConverter, NoteRenderer, NoteWriter, Problem, StatementNormalizer,
    extract_slug,
};

#[allow(async_fn_in_trait)]
pub trait Stage {
    type Input;
    type Output;

    async fn run(&self, input: Self::Input) -> Result<Self::Output>;
}

pub struct Pipeline<S> {
    stages: S,
}

impl<S: Stage> Pipeline<S> {
    pub fn new(stage: S) -> Self {
        Self { stages: stage }
    }

    pub fn then<N>(self, next: N) -> Pipeline<Then<S, N>>
    where
        N: Stage<Input = S::Output>,
    {
        Pipeline::new(Then {
            first: self.stages,
            next,
        })
    }

    pub async fn run(&self, input: S::Input) -> Result<S::Output> {
        self.stages.run(input).await
    }
}

pub struct Then<A, B> {
    first: A,
    next: B,
}

impl<A, B> Stage for Then<A, B>
where
    A: Stage,
    B: Stage<Input = A::Output>,
{
    type Input = A::Input;
    type Output = B::Output;

    async fn run(&self, input: Self::Input) -> Result<Self::Output> {
        let output = self.first.run(input).await?;
        self.next.run(output).await
    }
}

pub struct Statement {
    pub problem: Problem,
    pub markdown: String,
}

pub struct RenderedNote {
    pub problem: Problem,
    pub content: String,
}

pub struct FetchProblem {
    client: LeetcodeClient,
}

impl FetchProblem {
    pub fn new(client: LeetcodeClient) -> Self {
        Self { client }
    }
}

impl Stage for FetchProblem {
    type Input = String;
    type Output = Problem;

    async fn run(&self, url: String) -> Result<Problem> {
        let slug = extract_slug(&url).context("Invalid LeetCode problem URL")?;
        self.client
            .fetch_problem(&slug)
            .await
            .context("Failed to fetch problem")
    }
}

pub struct ConvertStatement {
    converter: MarkdownConverter,
}

impl ConvertStatement {
    pub fn new(converter: MarkdownConverter) -> Self {
        Self { converter }
    }
}

impl Stage for ConvertStatement {
    type Input = Problem;
    type Output = Statement;

    async fn run(&self, problem: Problem) -> Result<Statement> {
        let markdown = self
            .converter
            .convert(&problem.content)
            .context("Failed to convert problem statement to Markdown")?;
        Ok(Statement { problem, markdown })
    }
}

pub struct NormalizeStatement<N> {
    normalizer: N,
}

impl<N: StatementNormalizer> NormalizeStatement<N> {
    pub fn new(normalizer: N) -> Self {
        Self { normalizer }
    }
}

impl<N: StatementNormalizer> Stage for NormalizeStatement<N> {
    type Input = Statement;
    type Output = Statement;

    async fn run(&self, statement: Statement) -> Result<Statement> {
        let markdown = self
            .normalizer
            .normalize(&statement.markdown)
            .await
            .context("Failed to normalize mathematical expressions")?;
        Ok(Statement {
            problem: statement.problem,
            markdown,
        })
    }
}

pub struct RenderNote {
    created: NaiveDate,
}

impl RenderNote {
    pub fn new(created: NaiveDate) -> Self {
        Self { created }
    }
}

impl Stage for RenderNote {
    type Input = Statement;
    type Output = RenderedNote;

    async fn run(&self, statement: Statement) -> Result<RenderedNote> {
        let content = NoteRenderer::render(&statement.problem, &statement.markdown, self.created)
            .context("Failed to render Obsidian note")?;
        Ok(RenderedNote {
            problem: statement.problem,
            content,
        })
    }
}

pub struct WriteNote {
    writer: NoteWriter,
}

impl WriteNote {
    pub fn new(writer: NoteWriter) -> Self {
        Self { writer }
    }
}

impl Stage for WriteNote {
    type Input = RenderedNote;
    type Output = PathBuf;

    async fn run(&self, note: RenderedNote) -> Result<PathBuf> {
        self.writer
            .write(&note.problem, &note.content)
            .context("Failed to write Obsidian note")
    }
}
