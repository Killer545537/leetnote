use url::Url;

use super::error::{LeetcodeError, Result};

pub fn extract_slug(input: &str) -> Result<String> {
    let url = Url::parse(input).map_err(|err| LeetcodeError::InvalidUrl(err.to_string()))?;

    if url.scheme() != "https" {
        return Err(LeetcodeError::InvalidUrl("Expected HTTPS URL".into()));
    }

    if url.host_str() != Some("leetcode.com") {
        return Err(LeetcodeError::InvalidUrl(
            "Expected leetcode.com hostname".into(),
        ));
    }

    let segments: Vec<&str> = url
        .path_segments()
        .ok_or_else(|| LeetcodeError::InvalidUrl("Missing path".into()))?
        .filter(|segment| !segment.is_empty())
        .collect();

    if segments.len() < 2 || segments[0] != "problems" {
        return Err(LeetcodeError::InvalidUrl(
            "Expected /problems/{slug}/".into(),
        ));
    }

    let slug = segments[1];
    if !slug
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(LeetcodeError::InvalidUrl("Invalid problem slug".into()));
    }

    Ok(slug.to_owned())
}
