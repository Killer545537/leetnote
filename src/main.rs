use std::path::PathBuf;

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use leetnote::{
    FmNormalizer, LeetcodeClient, MarkdownConverter, NoteRenderer, NoteWriter, StatementNormalizer,
    extract_slug,
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

    // Extract problem slug from url
    let slug = extract_slug(&args.url)?;

    // Fetch problem metadata and HTML
    let client = LeetcodeClient::new()?;
    let problem = client.fetch_problem(&slug).await?;

    // Convert HTML statement to Markdown
    let converter = MarkdownConverter::new();
    let statement = converter.convert(&problem.content)?;

    // Normalize the statement using the Foundation Models API
    let schema_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas/statement-normalization.json");
    let normalizer = FmNormalizer::new(&schema_path);
    let statement = normalizer.normalize(&statement).await?;

    // Render the Obsidian note
    let created = Local::now().date_naive();
    let note = NoteRenderer::render(&problem, &statement, created)?;

    // Write the note without overwriting the existing files
    let writer = NoteWriter::new(args.output_dir);
    let path = writer.write(&problem, &note)?;

    println!("Note created: {}", path.display());

    Ok(())
}
