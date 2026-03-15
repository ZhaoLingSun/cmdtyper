//! Integration test: parse every TOML data file with the real Rust types.
//! Ensures zero deserialization errors across all content files.

use std::fs;
use std::path::Path;

use cmdtyper::data::models::{CommandFile, CommandLesson, SymbolTopic, SystemTopic};

fn data_dir() -> &'static Path {
    Path::new("data")
}

#[test]
fn parse_all_command_files() {
    let dir = data_dir().join("commands");
    let mut count = 0;
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("Cannot read {}: {e}", dir.display()))
    {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));
            let parsed: Result<CommandFile, _> = toml::from_str(&content);
            assert!(
                parsed.is_ok(),
                "Failed to parse {}: {}",
                path.display(),
                parsed.unwrap_err()
            );
            let cf = parsed.unwrap();
            assert!(
                !cf.commands.is_empty(),
                "{}: commands array is empty",
                path.display()
            );
            for cmd in &cf.commands {
                assert!(
                    !cmd.id.is_empty(),
                    "{}: command has empty id",
                    path.display()
                );
                assert!(
                    !cmd.command.is_empty(),
                    "{}: command '{}' has empty command field",
                    path.display(),
                    cmd.id
                );
                assert!(
                    !cmd.tokens.is_empty(),
                    "{}: command '{}' has no tokens",
                    path.display(),
                    cmd.id
                );
                assert!(
                    !cmd.dictation.prompt.is_empty(),
                    "{}: command '{}' has empty dictation prompt",
                    path.display(),
                    cmd.id
                );
                assert!(
                    !cmd.dictation.answers.is_empty(),
                    "{}: command '{}' has no dictation answers",
                    path.display(),
                    cmd.id
                );
            }
            count += 1;
        }
    }
    assert!(
        count >= 19,
        "Expected at least 19 command files, found {count}"
    );
    println!("Successfully parsed {count} command files");
}

#[test]
fn parse_all_lesson_files() {
    let dir = data_dir().join("lessons");
    let mut count = 0;
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("Cannot read {}: {e}", dir.display()))
    {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));
            let parsed: Result<CommandLesson, _> = toml::from_str(&content);
            assert!(
                parsed.is_ok(),
                "Failed to parse {}: {}",
                path.display(),
                parsed.unwrap_err()
            );
            let lesson = parsed.unwrap();
            assert!(
                !lesson.meta.command.is_empty(),
                "{}: lesson has empty command",
                path.display()
            );
            assert!(
                !lesson.examples.is_empty(),
                "{}: lesson '{}' has no examples",
                path.display(),
                lesson.meta.command
            );
            count += 1;
        }
    }
    assert!(
        count >= 31,
        "Expected at least 31 lesson files, found {count}"
    );
    println!("Successfully parsed {count} lesson files");
}

#[test]
fn parse_all_symbol_files() {
    let dir = data_dir().join("symbols");
    let mut count = 0;
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("Cannot read {}: {e}", dir.display()))
    {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));
            let parsed: Result<SymbolTopic, _> = toml::from_str(&content);
            assert!(
                parsed.is_ok(),
                "Failed to parse {}: {}",
                path.display(),
                parsed.unwrap_err()
            );
            let topic = parsed.unwrap();
            assert!(
                !topic.symbols.is_empty(),
                "{}: symbol topic has no symbols",
                path.display()
            );
            count += 1;
        }
    }
    assert!(
        count >= 6,
        "Expected at least 6 symbol files, found {count}"
    );
    println!("Successfully parsed {count} symbol files");
}

#[test]
fn parse_all_system_files() {
    let dir = data_dir().join("system");
    let mut count = 0;
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("Cannot read {}: {e}", dir.display()))
    {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));
            let parsed: Result<SystemTopic, _> = toml::from_str(&content);
            assert!(
                parsed.is_ok(),
                "Failed to parse {}: {}",
                path.display(),
                parsed.unwrap_err()
            );
            let topic = parsed.unwrap();
            assert!(
                !topic.sections.is_empty(),
                "{}: system topic has no sections",
                path.display()
            );
            count += 1;
        }
    }
    assert!(
        count >= 6,
        "Expected at least 6 system files, found {count}"
    );
    println!("Successfully parsed {count} system files");
}
