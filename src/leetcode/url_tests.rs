use super::extract_slug;

#[test]
fn extracts_slug_from_problem_url() {
    let url = "https://leetcode.com/problems/assign-cookies/";

    assert_eq!(extract_slug(url).unwrap(), "assign-cookies");
}

#[test]
fn extracts_slug_from_description_url() {
    let url = "https://leetcode.com/problems/assign-cookies/description/";

    assert_eq!(extract_slug(url).unwrap(), "assign-cookies");
}

#[test]
fn ignores_query_parameters() {
    let url = "https://leetcode.com/problems/two-sum/?envType=study-plan";

    assert_eq!(extract_slug(url).unwrap(), "two-sum");
}

#[test]
fn rejects_wrong_hostname() {
    let url = "https://example.com/problems/two-sum/";

    assert!(extract_slug(url).is_err());
}

#[test]
fn rejects_http() {
    let url = "http://leetcode.com/problems/two-sum/";

    assert!(extract_slug(url).is_err());
}

#[test]
fn rejects_invalid_path() {
    let url = "https://leetcode.com/contest/";

    assert!(extract_slug(url).is_err());
}

#[test]
fn rejects_invalid_slug() {
    let url = "https://leetcode.com/problems/Two-Sum/";

    assert!(extract_slug(url).is_err());
}
