use std::fs;
use std::path::Path;

use cmdtyper::data::models::{
    CommandFile, CommandLesson, Exercise, SymbolTopic, SystemCommand, SystemTopic,
    TypingDisplayMode, UserConfig,
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
