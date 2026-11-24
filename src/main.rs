use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
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
    /// Problem URL, e.g. https://leetcode.com/problems/two-sum/
    #[arg(required_unless_present = "daily")]
    url: Option<String>,

    /// Use today's daily challenge instead of a URL
    #[arg(long, conflicts_with = "url")]
    daily: bool,

    /// Directory where the note will be created (e.g. your Obsidian vault folder)
    #[arg(short, long, env = "LEETNOTE_DIR")]
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

    let client = LeetcodeClient::new()?;
    let url = match args.url {
        Some(url) => url,
        None => client.daily_problem_url().await.context("Failed to fetch today's daily problem")?,
    };

    let pipeline = Pipeline::new(FetchProblem::new(client))
        .then(ConvertStatement::new(MarkdownConverter::new()))
        .then(NormalizeStatement::new(normalizer))
        .then(RenderNote::new(Local::now().date_naive()))
        .then(WriteNote::new(NoteWriter::new(args.output_dir)));

    let path = pipeline.run(url).await?;

    println!("Note created: {}", path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needs_exactly_one_of_url_or_daily() {
        let parse = |args: &[&str]| Args::try_parse_from([&["leetnote", "-o", "notes"], args].concat());
        assert!(parse(&["--daily"]).unwrap().daily);
        assert!(parse(&["https://leetcode.com/problems/two-sum/"]).unwrap().url.is_some());
        assert!(parse(&[]).is_err());
        assert!(parse(&["--daily", "https://leetcode.com/problems/two-sum/"]).is_err());
    }
}
