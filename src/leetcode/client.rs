use reqwest::Client;
use serde::{Deserialize, de::DeserializeOwned};
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
const DAILY_QUERY: &str = "query { activeDailyCodingChallengeQuestion { link } }";

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DailyData {
    active_daily_coding_challenge_question: DailyQuestion,
}

#[derive(Debug, Deserialize)]
struct DailyQuestion {
    link: String,
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
        let data: QuestionData = self.query(QUESTION_QUERY, json!({ "titleSlug": slug })).await?;
        let question = data.question.ok_or_else(|| LeetcodeError::ProblemNotFound(slug.to_owned()))?;
        parse_question(question, slug)
    }

    /// URL of today's daily challenge, e.g. `https://leetcode.com/problems/two-sum/`.
    pub async fn daily_problem_url(&self) -> Result<String> {
        let data: DailyData = self.query(DAILY_QUERY, json!({})).await?;
        Ok(format!("https://leetcode.com{}", data.active_daily_coding_challenge_question.link))
    }

    async fn query<T: DeserializeOwned>(&self, query: &str, variables: Value) -> Result<T> {
        let response = self
            .client
            .post(GRAPHQL_ENDPOINT)
            .header("Referer", "https://leetcode.com/")
            .json(&json!({ "query": query, "variables": variables }))
            .send()
            .await?
            .error_for_status()?;

        let result: GraphQlResponse<T> = response.json().await?;

        if let Some(errors) = result.errors
            && !errors.is_empty()
        {
            let messages = errors.into_iter().map(|error| error.message).collect::<Vec<_>>().join("; ");
            return Err(LeetcodeError::GraphQL(messages));
        }

        result.data.ok_or_else(|| LeetcodeError::UnexpectedResponse("Missing GraphQL data".into()))
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

    #[test]
    fn daily_response_yields_the_problem_link() {
        let body = r#"{"data":{"activeDailyCodingChallengeQuestion":{"link":"/problems/two-sum/"}}}"#;
        let response: GraphQlResponse<DailyData> = serde_json::from_str(body).unwrap();
        assert_eq!(response.data.unwrap().active_daily_coding_challenge_question.link, "/problems/two-sum/");
    }
}
