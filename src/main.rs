use std::{fs, path::PathBuf};

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use leetnote::{
    FmNormalizer, LeetcodeClient, MarkdownConverter, NoteWriter,
    normalization::SCHEMA,
    pipeline::{ConvertStatement, FetchProblem, NormalizeStatement, Pipeline, RenderNote, WriteNote},
};

#[derive(Debug, Parser)]
#[command(version, about = "Generate Obsidian notes from Leetcode problems")]
struct Args {
    url: String,

    /// Directory where the note will be created
    #[arg(short, long)]
    output_dir: PathBuf,

    /// Keep math as LeetCode writes it instead of converting it to LaTeX with Apple's `fm` (macOS 27+)
    #[arg(long)]
    no_normalize: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // `fm --schema` takes a path; the temp file is deleted when `schema` drops at the end of main.
    let schema = tempfile::Builder::new().suffix(".json").tempfile()?;
    fs::write(schema.path(), SCHEMA)?;
    let normalizer = (!args.no_normalize).then(|| FmNormalizer::new(schema.path()));

    let pipeline = Pipeline::new(FetchProblem::new(LeetcodeClient::new()?))
        .then(ConvertStatement::new(MarkdownConverter::new()))
        .then(NormalizeStatement::new(normalizer))
        .then(RenderNote::new(Local::now().date_naive()))
        .then(WriteNote::new(NoteWriter::new(args.output_dir)));

    let path = pipeline.run(args.url).await?;

    println!("Note created: {}", path.display());

    Ok(())
}
