use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Problem {
    pub question_frontend_id: String,
    pub title: String,
    pub title_slug: String,
    pub difficulty: Difficulty,
    pub content: String,
    pub topic_tags: Vec<TopicTag>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopicTag {
    pub name: String,
    pub slug: String,
}
