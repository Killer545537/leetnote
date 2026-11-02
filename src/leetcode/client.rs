use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

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
    question: Option<Problem>,
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
            let messages = errors
                .into_iter()
                .map(|error| error.message)
                .collect::<Vec<_>>()
                .join("; ");
            return Err(LeetcodeError::GraphQL(messages));
        }

        let data = result
            .data
            .ok_or_else(|| LeetcodeError::UnexpectedResponse("Missing GraphQL data".into()))?;

        data.question
            .ok_or_else(|| LeetcodeError::ProblemNotFound(slug.to_owned()))
    }
}
