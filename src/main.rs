mod error;
mod leetcode;

use anyhow::{Context, Result};
use leetcode::{LeetcodeClient, extract_slug};

#[tokio::main]
async fn main() -> Result<()> {
    let url = std::env::args()
        .nth(1)
        .context("Usage: leetnote <leetcode-url>")?;

    let slug = extract_slug(&url)?;

    let client = LeetcodeClient::new()?;

    let problem = client.fetch_problem(&slug).await?;

    println!("ID: {}", problem.question_frontend_id);
    println!("Title: {}", problem.title);
    println!("Difficulty: {:?}", problem.difficulty);

    println!("\nTags:");

    for tag in &problem.topic_tags {
        println!("- {} ({})", tag.name, tag.slug);
    }

    println!("\nStatement:\n{}", problem.content);

    Ok(())
}
