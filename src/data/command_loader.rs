use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::data::models::{Category, Command, CommandFile, Difficulty};

/// Load all commands from `data_dir/commands/*.toml`, propagating file-level
/// metadata (category, difficulty) into each individual [`Command`].
pub fn load_commands(data_dir: &Path) -> Result<Vec<Command>> {
    let commands_dir = data_dir.join("commands");
    let mut all_commands = Vec::new();

    if !commands_dir.exists() {
        return Ok(all_commands);
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(&commands_dir)
        .with_context(|| format!("failed to read commands directory {}", commands_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect();

    entries.sort();

    for path in entries {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let file: CommandFile = toml::from_str(&contents)
            .with_context(|| format!("failed to parse {}", path.display()))?;

        let category = file.meta.category;
        let difficulty = file.meta.difficulty;

        for mut cmd in file.commands {
            cmd.category = category;
            cmd.difficulty = difficulty;
            all_commands.push(cmd);
        }
    }

    Ok(all_commands)
}

/// Filter commands by difficulty.
pub fn load_by_difficulty(commands: &[Command], difficulty: Difficulty) -> Vec<Command> {
    commands
        .iter()
        .filter(|cmd| cmd.difficulty == difficulty)
        .cloned()
        .collect()
}

/// Filter commands by category.
pub fn load_by_category(commands: &[Command], category: Category) -> Vec<Command> {
    commands
        .iter()
        .filter(|cmd| cmd.category == category)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_data_dir() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cmdtyper-cmd-test-{suffix}"));
        fs::create_dir_all(dir.join("commands")).expect("temp dir should be created");
        dir
    }

    #[test]
    fn load_commands_propagates_metadata() {
        let dir = temp_data_dir();
        let fixture = r#"
[meta]
category = "search"
difficulty = "advanced"
description = "Search commands"

[[commands]]
id = "grep-basic"
command = "grep foo file.txt"
summary = "Search for foo"
tokens = []

[commands.dictation]
prompt = "Search for foo in file.txt"
answers = ["grep foo file.txt"]
"#;

        fs::write(dir.join("commands/search.toml"), fixture).expect("fixture should write");

        let commands = load_commands(&dir).expect("should load commands");
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].id, "grep-basic");
        assert_eq!(commands[0].category, Category::Search);
        assert_eq!(commands[0].difficulty, Difficulty::Advanced);

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn missing_dir_returns_empty() {
        let dir = Path::new("/tmp/nonexistent-cmdtyper-dir-99999");
        let commands = load_commands(dir).expect("should not error on missing dir");
        assert!(commands.is_empty());
    }

    #[test]
    fn v02_optional_fields_have_defaults() {
        let dir = temp_data_dir();
        let fixture = r#"
[meta]
category = "file_ops"
difficulty = "beginner"
description = "File ops"

[[commands]]
id = "ls-basic"
command = "ls"
summary = "list files"
display = "ls -la"
summary_short = "list"
simulated_output = "total 0"
tokens = []

[[commands.output_annotations]]
pattern = "total"
note = "total size"

[commands.dictation]
prompt = "list files"
answers = ["ls"]
"#;

        fs::write(dir.join("commands/file_ops.toml"), fixture).expect("fixture should write");

        let commands = load_commands(&dir).expect("should load");
        assert_eq!(commands[0].display.as_deref(), Some("ls -la"));
        assert_eq!(commands[0].summary_short.as_deref(), Some("list"));
        assert_eq!(commands[0].simulated_output.as_deref(), Some("total 0"));
        assert_eq!(commands[0].output_annotations.len(), 1);
        assert_eq!(commands[0].display_text(), "ls -la");
        assert_eq!(commands[0].short_summary(), "list");

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn filters_work() {
        let dir = temp_data_dir();
        let fixture1 = r#"
[meta]
category = "search"
difficulty = "advanced"
description = "Search"

[[commands]]
id = "grep-basic"
command = "grep foo"
summary = "grep"
tokens = []
[commands.dictation]
prompt = "grep"
answers = ["grep foo"]
"#;
        let fixture2 = r#"
[meta]
category = "archive"
difficulty = "beginner"
description = "Archive"

[[commands]]
id = "tar-basic"
command = "tar -tf a.tar"
summary = "tar"
tokens = []
[commands.dictation]
prompt = "tar"
answers = ["tar -tf a.tar"]
"#;

        fs::write(dir.join("commands/search.toml"), fixture1).expect("write");
        fs::write(dir.join("commands/archive.toml"), fixture2).expect("write");

        let commands = load_commands(&dir).expect("load");
        assert_eq!(load_by_difficulty(&commands, Difficulty::Beginner).len(), 1);
        assert_eq!(load_by_category(&commands, Category::Search).len(), 1);
        assert!(load_by_category(&commands, Category::FileOps).is_empty());

        fs::remove_dir_all(dir).expect("cleanup");
    }
}
