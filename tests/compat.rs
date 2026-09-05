use std::fs;
use std::path::Path;

use cmdtyper::data::models::{
    CommandFile, CommandLesson, Exercise, LessonExample, ResumeScreen, ResumeState, SymbolExample,
    SymbolTopic, SystemCommand, SystemTopic, TopicTrainingLevel, TypingDisplayMode, UserConfig,
};

fn data_dir() -> &'static Path {
    Path::new("data")
}

#[test]
fn existing_toml_files_still_parse() {
    for sub in ["commands", "lessons", "symbols", "system"] {
        let dir = data_dir().join(sub);
        for entry in
            fs::read_dir(&dir).unwrap_or_else(|e| panic!("Cannot read {}: {e}", dir.display()))
        {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
                continue;
            }
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));

            let ok = match sub {
                "commands" => toml::from_str::<CommandFile>(&content).is_ok(),
                "lessons" => toml::from_str::<CommandLesson>(&content).is_ok(),
                "symbols" => toml::from_str::<SymbolTopic>(&content).is_ok(),
                "system" => toml::from_str::<SystemTopic>(&content).is_ok(),
                _ => false,
            };
            assert!(ok, "Failed to parse {}", path.display());
        }
    }
}

#[test]
fn old_command_toml_defaults_to_no_training_topic() {
    let file: CommandFile = toml::from_str(
        r#"
[meta]
category = "file_ops"
difficulty = "beginner"
description = "legacy"

[[commands]]
id = "ls-basic"
command = "ls"
summary = "list"
tokens = []
[commands.dictation]
prompt = "list"
answers = ["ls"]
"#,
    )
    .expect("legacy command file should parse");

    assert!(file.meta.topic.is_none());
}

#[test]
fn command_file_parses_one_topic_from_meta() {
    let file: CommandFile = toml::from_str(
        r#"
[meta]
category = "file_ops"
difficulty = "beginner"
description = "basics"

[meta.topic]
id = "file-basics"
title = "File Basics"
icon = "files"
order = 3

[[commands]]
id = "ls-basic"
command = "ls"
summary = "list"
tokens = []
[commands.dictation]
prompt = "list"
answers = ["ls"]
"#,
    )
    .expect("command file topic should parse");

    let topic = file.meta.topic.expect("topic metadata should exist");
    assert_eq!(topic.id, "file-basics");
    assert_eq!(topic.title, "File Basics");
    assert_eq!(topic.icon.as_deref(), Some("files"));
    assert_eq!(topic.order, 3);
}

#[test]
fn reference_only_content_records_deserialize_with_rendered_defaults() {
    let lesson: LessonExample = toml::from_str(
        r#"
id = "lesson-ls"
command_id = "ls-basic"
"#,
    )
    .expect("reference-only lesson example should parse");
    assert_eq!(lesson.level, 0);
    assert!(lesson.command.is_empty());
    assert!(lesson.summary.is_empty());
    assert!(lesson.token_details.is_empty());

    let symbol: SymbolExample = toml::from_str(
        r#"
id = "symbol-ls"
command_id = "ls-basic"
"#,
    )
    .expect("reference-only symbol example should parse");
    assert!(symbol.command.is_empty());
    assert!(symbol.explanation.is_empty());

    let exercise: Exercise = toml::from_str(
        r#"
id = "exercise-ls"
command_id = "ls-basic"
"#,
    )
    .expect("reference-only exercise should parse");
    assert!(exercise.prompt.is_empty());
    assert!(exercise.answers.is_empty());
    assert_eq!(exercise.command, None);

    let system: SystemCommand = toml::from_str(
        r#"
id = "system-ls"
command_id = "ls-basic"
"#,
    )
    .expect("reference-only system command should parse");
    assert!(system.command.is_empty());
    assert!(system.summary.is_empty());
}

#[test]
fn deep_explanation_defaults_to_none() {
    let lesson: CommandLesson = toml::from_str(
        r#"
[meta]
command = "ls"
category = "file_ops"
difficulty = "beginner"

[overview]
summary = "list"
explanation = "list files"

[syntax]
basic = "ls"

[[examples]]
level = 1
command = "ls"
summary = "basic"
"#,
    )
    .expect("lesson parse");
    assert_eq!(lesson.examples[0].deep_explanation, None);

    let symbol: SymbolTopic = toml::from_str(
        r#"
[meta]
id = "pipe"
topic = "pipe"
description = "desc"
difficulty = "basic"

[[symbols]]
id = "s1"
char_repr = "|"
name = "pipe"
summary = "sum"
explanation = "exp"

[[symbols.examples]]
command = "ls | wc -l"
explanation = "pipe"
"#,
    )
    .expect("symbol parse");
    assert_eq!(symbol.symbols[0].examples[0].deep_explanation, None);

    let sys_cmd: SystemCommand = toml::from_str(
        r#"
command = "systemctl status"
summary = "status"
"#,
    )
    .expect("system command parse");
    assert_eq!(sys_cmd.deep_explanation, None);
}

#[test]
fn exercise_kind_defaults_to_none() {
    let ex: Exercise = toml::from_str(
        r#"
prompt = "type ls"
answers = ["ls"]
"#,
    )
    .expect("exercise parse");

    assert_eq!(ex.kind, None);
    assert_eq!(ex.command, None);
    assert_eq!(ex.simulated_output, None);
}

#[test]
fn old_resume_json_defaults_new_stable_identifiers() {
    let resume: ResumeState = serde_json::from_str(
        r#"{
            "screen": "symbol_example",
            "category_index": 1,
            "command_index": 2,
            "example_index": 3,
            "topic_index": 4,
            "symbol_index": 5,
            "section_index": 6,
            "overview_scroll": 7
        }"#,
    )
    .expect("legacy resume JSON should parse");

    assert_eq!(resume.screen, ResumeScreen::SymbolExample);
    assert_eq!(resume.topic_index, 4);
    assert_eq!(resume.example_index, 3);
    assert_eq!(resume.overview_scroll, 7);
    assert_eq!(resume.lesson_command, None);
    assert_eq!(resume.symbol_topic_id, None);
    assert_eq!(resume.symbol_id, None);
    assert_eq!(resume.system_topic_id, None);
    assert_eq!(resume.system_section_id, None);
    assert_eq!(resume.review_topic_id, None);
    assert_eq!(resume.topic_training_level, TopicTrainingLevel::L1);
}

#[test]
fn topic_training_level_round_trips_in_review_summary_resume() {
    let resume = ResumeState {
        screen: ResumeScreen::ReviewSummary,
        review_topic_id: Some("help_rescue".to_string()),
        topic_training_level: TopicTrainingLevel::L5,
        ..ResumeState::default()
    };
    let json = serde_json::to_string(&resume).expect("serialize review resume");
    assert!(json.contains("\"topic_training_level\":\"l5\""));
    let restored: ResumeState = serde_json::from_str(&json).expect("deserialize review resume");
    assert_eq!(restored, resume);
}

#[test]
fn user_config_typing_mode_defaults_to_standard() {
    let cfg: UserConfig = toml::from_str(
        r#"
target_wpm = 50.0
error_flash_ms = 120
show_token_hints = true
adaptive_recommend = true
last_difficulty = "beginner"
last_category = "file_ops"
"#,
    )
    .expect("config parse");

    assert_eq!(cfg.typing_mode, TypingDisplayMode::Standard);
}
