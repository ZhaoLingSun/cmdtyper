use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::data::models::{Category, Command, CommandFile, Difficulty};

pub fn load_commands(data_dir: &Path) -> Result<Vec<Command>> {
    let mut all_commands = Vec::new();

    if !data_dir.exists() {
        return Ok(all_commands);
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(data_dir)
        .with_context(|| format!("failed to read commands directory {}", data_dir.display()))?
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

pub fn load_by_difficulty(commands: &[Command], difficulty: Difficulty) -> Vec<Command> {
    commands
        .iter()
        .filter(|command| command.difficulty == difficulty)
        .cloned()
        .collect()
}

pub fn load_by_category(commands: &[Command], category: Category) -> Vec<Command> {
    commands
        .iter()
        .filter(|command| command.category == category)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{load_by_category, load_by_difficulty, load_commands};
    use crate::data::models::{Category, Difficulty};
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_commands_dir() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cmdtyper-loader-test-{suffix}"));
        fs::create_dir_all(&dir).expect("temp command dir should be created");
        dir
    }

    #[test]
    fn load_commands_from_toml_files_propagates_metadata() {
        let dir = temp_commands_dir();
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

        fs::write(dir.join("search.toml"), fixture).expect("fixture should be written");

        let commands = load_commands(&dir).expect("should load commands");

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].id, "grep-basic");
        assert_eq!(commands[0].category, Category::Search);
        assert_eq!(commands[0].difficulty, Difficulty::Advanced);

        fs::remove_dir_all(dir).expect("temp command dir should be removable");
    }

    #[test]
    fn load_commands_missing_dir_returns_empty() {
        let dir = Path::new("/tmp/nonexistent-cmdtyper-dir-12345");
        let commands = load_commands(dir).expect("should not error on missing dir");
        assert!(commands.is_empty());
    }

    #[test]
    fn load_filters_work_on_loaded_commands() {
        let dir = temp_commands_dir();
        let search_fixture = r#"
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
        let archive_fixture = r#"
[meta]
category = "archive"
difficulty = "beginner"
description = "Archive commands"

[[commands]]
id = "tar-basic"
command = "tar -tf backup.tar"
summary = "List archive contents"
tokens = []

[commands.dictation]
prompt = "List archive contents"
answers = ["tar -tf backup.tar"]
"#;

        fs::write(dir.join("search.toml"), search_fixture).expect("search fixture should write");
        fs::write(dir.join("archive.toml"), archive_fixture).expect("archive fixture should write");

        let commands = load_commands(&dir).expect("should load commands");

        let beginner = load_by_difficulty(&commands, Difficulty::Beginner);
        let file_ops = load_by_category(&commands, Category::FileOps);
        let search = load_by_category(&commands, Category::Search);

        assert!(beginner
            .iter()
            .all(|command| command.difficulty == Difficulty::Beginner));
        assert_eq!(beginner.len(), 1);
        assert!(file_ops
            .iter()
            .all(|command| command.category == Category::FileOps));
        assert!(file_ops.is_empty());
        assert_eq!(search.len(), 1);
        assert_eq!(search[0].id, "grep-basic");

        fs::remove_dir_all(dir).expect("temp command dir should be removable");
    }
}
