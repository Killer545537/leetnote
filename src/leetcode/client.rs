use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::leetcode::models::Problem;

use super::error::{LeetcodeError, Result};

const GRAPHQL_ENDPOINT: &str = "https://leetcode.com/graphql";
const QUESTION_QUERY: &str = r#"
    query questionData($titleSlug: String!) {
        question(titleSlug: $titleSlug) {
            questionFrontendId
            title
            titleSlug
            difficulty
            content
            topicTags {
                name
                slug
            }
        }
    }
"#;

#[derive(Debug, Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQlError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct QuestionData {
    // Raw JSON so a premium problem's `content: null` gives a clear error instead of a decode failure.
    question: Option<Value>,
}

pub struct LeetcodeClient {
    client: Client,
}

impl LeetcodeClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder().user_agent("leetnote/0.1.0").build()?;

        Ok(Self { client })
    }

    pub async fn fetch_problem(&self, slug: &str) -> Result<Problem> {
        let response = self
            .client
            .post(GRAPHQL_ENDPOINT)
            .header("Referer", "https://leetcode.com/")
            .json(&json!({
                "query": QUESTION_QUERY,
                "variables": {
                    "titleSlug": slug
                }
            }))
            .send()
            .await?
            .error_for_status()?;

        let result: GraphQlResponse<QuestionData> = response.json().await?;

        if let Some(errors) = result.errors
            && !errors.is_empty()
        {
            let messages = errors.into_iter().map(|error| error.message).collect::<Vec<_>>().join("; ");
            return Err(LeetcodeError::GraphQL(messages));
        }

        let data = result.data.ok_or_else(|| LeetcodeError::UnexpectedResponse("Missing GraphQL data".into()))?;

        let question = data.question.ok_or_else(|| LeetcodeError::ProblemNotFound(slug.to_owned()))?;
        parse_question(question, slug)
    }
}

fn parse_question(question: Value, slug: &str) -> Result<Problem> {
    if question["content"].is_null() {
        return Err(LeetcodeError::PremiumOnly(slug.to_owned()));
    }
    serde_json::from_value(question).map_err(|err| LeetcodeError::UnexpectedResponse(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premium_problem_without_content_is_a_clear_error() {
        let question = json!({
            "questionFrontendId": "252", "title": "Meeting Rooms", "titleSlug": "meeting-rooms",
            "difficulty": "Easy", "content": null, "topicTags": [],
        });
        let err = parse_question(question.clone(), "meeting-rooms").unwrap_err();
        assert!(matches!(err, LeetcodeError::PremiumOnly(ref slug) if slug == "meeting-rooms"));

        let mut free = question;
        free["content"] = json!("<p>Statement</p>");
        assert_eq!(parse_question(free, "meeting-rooms").unwrap().content, "<p>Statement</p>");
    }
}
