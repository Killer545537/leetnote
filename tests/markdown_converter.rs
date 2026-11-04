use leetnote::MarkdownConverter;

const ASSIGN_COOKIES_HTML: &str = include_str!("fixtures/assign_cookies.html");

#[test]
fn converts_assign_cookies_statement() {
    let converter = MarkdownConverter::new();
    let markdown = converter.convert(ASSIGN_COOKIES_HTML).unwrap();

    println!("{markdown}");

    // Statement content
    assert!(markdown.contains("awesome parent"));
    assert!(markdown.contains("cookies"));

    // Examples
    assert!(markdown.contains("Input:"));
    assert!(markdown.contains("Output:"));

    // Constraints
    assert!(markdown.contains("Constraints"));

    // No image markup or image URLs
    assert!(!markdown.contains("<img"));
}
