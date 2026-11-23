use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use super::error::Result;
use crate::{leetcode::Problem, obsidian::ObsidianError};

pub struct NoteWriter {
    directory: PathBuf,
}

impl NoteWriter {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self { directory: directory.into() }
    }

    pub fn write(&self, problem: &Problem, content: &str) -> Result<PathBuf> {
        fs::create_dir_all(&self.directory)
            .map_err(|source| ObsidianError::Io { path: self.directory.clone(), source })?;

        let path = self.directory.join(format!("{}.md", Self::sanitize_filename(&problem.title)));

        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(ObsidianError::AlreadyExists(path));
            }
            Err(source) => return Err(ObsidianError::Io { path, source }),
        };

        if let Err(source) = file.write_all(content.as_bytes()) {
            drop(file);
            let _ = fs::remove_file(&path);

            return Err(ObsidianError::Io { path, source });
        }

        Ok(path)
    }

    fn sanitize_filename(input: &str) -> String {
        let sanitized: String = input
            .chars()
            .map(|character| {
                if character.is_control() || matches!(character, '/' | '\\' | '<' | '>' | ':' | '"' | '|' | '?' | '*') {
                    ' '
                } else {
                    character
                }
            })
            .collect();

        sanitized.split_whitespace().collect::<Vec<_>>().join(" ").trim_matches([' ', '.']).to_owned()
    }
}
