mod error;
mod leetcode;
mod markdown;
mod obsidian;

use anyhow::{Context, Ok, Result};
use chrono::Local;
use leetcode::{LeetcodeClient, extract_slug};

use clap::Parser;
use leetnote::MarkdownConverter;

use crate::obsidian::NoteRenderer;

#[derive(Debug, Parser)]
#[command(version, about = "Generate Obsidian notes from Leetcode problems")]
struct Args {
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let slug = extract_slug(&args.url)?;

    let client = LeetcodeClient::new()?;
    let problem = client.fetch_problem(&slug).await?;

    let converter = MarkdownConverter::new();
    let statement = converter.convert(&problem.content)?;

    let created = Local::now().date_naive();

    let note = NoteRenderer::render(&problem, &statement, created)?;

    println!("{note}");

    Ok(())
}
