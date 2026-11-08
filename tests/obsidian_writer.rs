use std::fs;

use leetnote::{Difficulty, NoteWriter, ObsidianError, Problem, TopicTag};

fn sample_problem() -> Problem {
    Problem {
        question_frontend_id: "455".into(),
        title: "Assign Cookies".into(),
        title_slug: "assign-cookies".into(),
        difficulty: Difficulty::Easy,
        content: String::new(),
        topic_tags: vec![TopicTag {
            name: "Greedy".into(),
            slug: "greedy".into(),
        }],
    }
}

#[test]
fn writes_note_to_directory() {
    let directory = tempfile::tempdir().unwrap();
    let writer = NoteWriter::new(directory.path());

    let path = writer.write(&sample_problem(), "# Assign Cookies").unwrap();

    assert_eq!(path.file_name().unwrap(), "Assign Cookies.md");

    assert_eq!(fs::read_to_string(path).unwrap(), "# Assign Cookies");
}

#[test]
fn refuses_to_overwrite_existing_note() {
    let directory = tempfile::tempdir().unwrap();
    let writer = NoteWriter::new(directory.path());

    let existing_path = directory.path().join("Assign Cookies.md");

    fs::write(&existing_path, "My handwritten notes").unwrap();

    let result = writer.write(&sample_problem(), "New generated content");

    assert!(matches!(result, Err(ObsidianError::AlreadyExists(_))));

    assert_eq!(
        fs::read_to_string(existing_path).unwrap(),
        "My handwritten notes"
    );
}

#[test]
fn creates_missing_directory() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("LeetCode");

    let writer = NoteWriter::new(&output);

    let path = writer
        .write(&sample_problem(), "Problem statement")
        .unwrap();

    assert!(path.exists());
}

#[test]
fn sanitizes_filename() {
    let directory = tempfile::tempdir().unwrap();
    let writer = NoteWriter::new(directory.path());

    let mut problem = sample_problem();
    problem.title = "Problem: A/B?".into();

    let path = writer.write(&problem, "Content").unwrap();

    assert_eq!(path.file_name().unwrap(), "Problem A B.md");
}
