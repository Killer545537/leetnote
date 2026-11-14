use std::path::PathBuf;

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use leetnote::{
    FmNormalizer, LeetcodeClient, MarkdownConverter, NoteWriter,
    pipeline::{
        ConvertStatement, FetchProblem, NormalizeStatement, Pipeline, RenderNote, WriteNote,
    },
};

#[derive(Debug, Parser)]
#[command(version, about = "Generate Obsidian notes from Leetcode problems")]
struct Args {
    url: String,

    /// Directory where the note will be created
    #[arg(short, long)]
    output_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let schema_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas/statement-normalization.json");

    let pipeline = Pipeline::new(FetchProblem::new(LeetcodeClient::new()?))
        .then(ConvertStatement::new(MarkdownConverter::new()))
        .then(NormalizeStatement::new(FmNormalizer::new(schema_path)))
        .then(RenderNote::new(Local::now().date_naive()))
        .then(WriteNote::new(NoteWriter::new(args.output_dir)));

    let path = pipeline.run(args.url).await?;

    println!("Note created: {}", path.display());

    Ok(())
}
