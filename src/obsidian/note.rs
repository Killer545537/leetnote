use chrono::NaiveDate;
use serde::Serialize;

use super::error::Result;
use crate::leetcode::{Difficulty, Problem};

#[derive(Debug, Serialize)]
struct FrontMatter<'a> {
    created: NaiveDate,
    number: &'a str,
    tags: Vec<&'a str>,
    difficulty: &'a str,
    url: String,
    #[serde(rename = "type")]
    note_type: &'static str,
    status: &'static str,
}

pub struct NoteRenderer;

impl NoteRenderer {
    pub fn render(problem: &Problem, statement: &str, created: NaiveDate) -> Result<String> {
        let frontmatter = FrontMatter {
            created,
            number: &problem.question_frontend_id,
            tags: problem
                .topic_tags
                .iter()
                .map(|tag| tag.slug.as_str())
                .collect(),
            difficulty: Self::difficulty_name(&problem.difficulty),
            url: format!(
                "https://leetcode.com/problems/{}/description/",
                problem.title_slug
            ),
            note_type: "problem",
            status: "todo",
        };

        let yaml = serde_yaml::to_string(&frontmatter)?;

        let mut note = String::new();

        note.push_str("---\n");
        note.push_str(&yaml);
        note.push_str("---\n\n");

        note.push_str("> [!problem]\n");
        note.push_str(&format!("> **{}**\n>\n", problem.title));

        for line in statement.lines() {
            if line.is_empty() {
                note.push_str(">\n");
            } else {
                note.push_str(&format!("> {}\n", line));
            }
        }

        note.push_str("\n## Observations\n");
        note.push_str("\n## Key Insight\n");
        note.push_str("\n## Algorithm\n");

        Ok(note)
    }

    fn difficulty_name(difficulty: &Difficulty) -> &str {
        match difficulty {
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leetcode::{Difficulty, Problem, TopicTag};

    fn test_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 4).unwrap()
    }

    fn sample_problem() -> Problem {
        Problem {
            question_frontend_id: "455".into(),
            title: "Assign Cookies".into(),
            title_slug: "assign-cookies".into(),
            difficulty: Difficulty::Easy,
            content: String::new(),
            topic_tags: vec![
                TopicTag {
                    name: "Array".into(),
                    slug: "array".into(),
                },
                TopicTag {
                    name: "Greedy".into(),
                    slug: "greedy".into(),
                },
            ],
        }
    }

    #[test]
    fn renders_frontmatter() {
        let note =
            NoteRenderer::render(&sample_problem(), "Problem statement.", test_date()).unwrap();

        assert!(note.contains("number: '455'"));
        assert!(note.contains("difficulty: easy"));
        assert!(note.contains("type: problem"));
        assert!(note.contains("status: todo"));
        assert!(note.contains("- array"));
        assert!(note.contains("- greedy"));
    }

    #[test]
    fn renders_problem_callout() {
        let note = NoteRenderer::render(
            &sample_problem(),
            "First paragraph.\n\nSecond paragraph.",
            test_date(),
        )
        .unwrap();

        assert!(note.contains("> [!problem]"));
        assert!(note.contains("> **Assign Cookies**"));
        assert!(note.contains("> First paragraph."));
        assert!(note.contains(">\n> Second paragraph."));
    }

    #[test]
    fn renders_empty_solution_sections() {
        let note =
            NoteRenderer::render(&sample_problem(), "Problem statement.", test_date()).unwrap();

        assert!(note.contains("## Observations"));
        assert!(note.contains("## Key Insight"));
        assert!(note.contains("## Algorithm"));
    }
}
