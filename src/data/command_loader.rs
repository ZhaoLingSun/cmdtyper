use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::data::catalog::validate_canonical_commands;
use crate::data::models::{
    Category, Command, CommandCatalog, CommandFile, CommandTrainingTopic, Difficulty,
};

/// Load all commands and optional one-topic-per-file metadata from
/// `data_dir/commands/*.toml`.
pub fn load_command_catalog(data_dir: &Path) -> Result<CommandCatalog> {
    let commands_dir = data_dir.join("commands");
    let mut catalog = CommandCatalog::default();

    if !commands_dir.exists() {
        return Ok(catalog);
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(&commands_dir)
        .with_context(|| {
            format!(
                "failed to read commands directory {}",
                commands_dir.display()
            )
        })?
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
        let mut command_ids: Vec<String> = file
            .commands
            .iter()
            .map(|command| command.id.clone())
            .collect();

        if let Some(topic) = file.meta.topic {
            for id in &topic.command_ids {
                if !command_ids.contains(id) {
                    command_ids.push(id.clone());
                }
            }
            catalog.topics.push(CommandTrainingTopic {
                id: topic.id,
                title: topic.title,
                icon: topic.icon,
                order: topic.order,
                description: file.meta.description,
                category,
                difficulty,
                command_ids,
            });
        }

        for mut command in file.commands {
            command.category = category;
            command.difficulty = difficulty;
            catalog.commands.push(command);
        }
    }

    validate_canonical_commands(&catalog.commands)?;
    validate_topics(&catalog.topics)?;
    let known: HashSet<_> = catalog.commands.iter().map(|c| c.id.as_str()).collect();
    for topic in &catalog.topics {
        for id in &topic.command_ids {
            if !known.contains(id.as_str()) {
                bail!("topic {} references unknown command {}", topic.id, id);
            }
        }
    }
    catalog.topics.sort_by_key(|topic| topic.order);
    Ok(catalog)
}

/// Load the same flat command list exposed before topic metadata existed.
pub fn load_commands(data_dir: &Path) -> Result<Vec<Command>> {
    Ok(load_command_catalog(data_dir)?.commands)
}

fn validate_topics(topics: &[CommandTrainingTopic]) -> Result<()> {
    let mut topic_ids = HashSet::new();
    let mut topic_orders = HashMap::new();

    for topic in topics {
        if topic.id.trim().is_empty() {
            bail!("command training topic ID must not be empty");
        }
        if topic.title.trim().is_empty() {
            bail!("command training topic {:?} has an empty title", topic.id);
        }
        if !topic_ids.insert(topic.id.as_str()) {
            bail!("duplicate command training topic ID {:?}", topic.id);
        }
        if let Some(previous_id) = topic_orders.insert(topic.order, topic.id.as_str()) {
            bail!(
                "duplicate command training topic order {} for {:?} and {:?}",
                topic.order,
                previous_id,
                topic.id
            );
        }
        if topic.command_ids.is_empty() {
            bail!(
                "command training topic {:?} must contain at least one command",
                topic.id
            );
        }
    }

    Ok(())
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

    fn command_fixture(id: &str, command: &str) -> String {
        format!(
            r#"
[meta]
category = "search"
difficulty = "advanced"
description = "Search commands"

[[commands]]
id = "{id}"
command = "{command}"
summary = "Search"
tokens = []

[commands.dictation]
prompt = "Search"
answers = ["{command}"]
"#
        )
    }

    #[test]
    fn load_commands_propagates_metadata() {
        let dir = temp_data_dir();
        fs::write(
            dir.join("commands/search.toml"),
            command_fixture("grep-basic", "grep foo file.txt"),
        )
        .expect("fixture should write");

        let commands = load_commands(&dir).expect("should load commands");
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].id, "grep-basic");
        assert_eq!(commands[0].category, Category::Search);
        assert_eq!(commands[0].difficulty, Difficulty::Advanced);

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn meta_topic_derives_runtime_topic_and_file_command_mapping() {
        let dir = temp_data_dir();
        let fixture = r#"
[meta]
category = "search"
difficulty = "advanced"
description = "Search commands"

[meta.topic]
id = "search-tools"
title = "Search Tools"
icon = "search-icon"
order = 20

[[commands]]
id = "grep-basic"
command = "grep foo file.txt"
summary = "Search"
tokens = []
[commands.dictation]
prompt = "Search"
answers = ["grep foo file.txt"]

[[commands]]
id = "find-basic"
command = "find . -name foo"
summary = "Find"
tokens = []
[commands.dictation]
prompt = "Find"
answers = ["find . -name foo"]
"#;
        fs::write(dir.join("commands/search.toml"), fixture).expect("fixture should write");

        let catalog = load_command_catalog(&dir).expect("catalog should load");
        assert_eq!(catalog.topics.len(), 1);
        let topic = &catalog.topics[0];
        assert_eq!(topic.id, "search-tools");
        assert_eq!(topic.title, "Search Tools");
        assert_eq!(topic.icon.as_deref(), Some("search-icon"));
        assert_eq!(topic.order, 20);
        assert_eq!(topic.description, "Search commands");
        assert_eq!(topic.category, Category::Search);
        assert_eq!(topic.difficulty, Difficulty::Advanced);
        assert_eq!(topic.command_ids, ["grep-basic", "find-basic"]);

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn topics_are_sorted_by_order_across_files() {
        let dir = temp_data_dir();
        let later = command_fixture("grep-basic", "grep foo").replace(
            "description = \"Search commands\"",
            "description = \"Search commands\"\n\n[meta.topic]\nid = \"later\"\ntitle = \"Later\"\norder = 20",
        );
        let earlier = command_fixture("find-basic", "find .").replace(
            "description = \"Search commands\"",
            "description = \"Search commands\"\n\n[meta.topic]\nid = \"earlier\"\ntitle = \"Earlier\"\norder = 10",
        );
        fs::write(dir.join("commands/a.toml"), later).expect("write");
        fs::write(dir.join("commands/b.toml"), earlier).expect("write");

        let catalog = load_command_catalog(&dir).expect("catalog should load");
        assert_eq!(catalog.topics[0].id, "earlier");
        assert_eq!(catalog.topics[1].id, "later");

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn duplicate_topic_id_and_order_are_rejected() {
        let dir = temp_data_dir();
        let first = command_fixture("grep-basic", "grep foo").replace(
            "description = \"Search commands\"",
            "description = \"Search commands\"\n\n[meta.topic]\nid = \"search\"\ntitle = \"Search\"\norder = 1",
        );
        let duplicate_id = command_fixture("find-basic", "find .").replace(
            "description = \"Search commands\"",
            "description = \"Search commands\"\n\n[meta.topic]\nid = \"search\"\ntitle = \"Find\"\norder = 2",
        );
        fs::write(dir.join("commands/a.toml"), &first).expect("write");
        fs::write(dir.join("commands/b.toml"), duplicate_id).expect("write");
        let error = load_command_catalog(&dir).expect_err("duplicate topic ID should fail");
        assert!(
            error
                .to_string()
                .contains("duplicate command training topic ID")
        );

        let duplicate_order = command_fixture("find-basic", "find .").replace(
            "description = \"Search commands\"",
            "description = \"Search commands\"\n\n[meta.topic]\nid = \"find\"\ntitle = \"Find\"\norder = 1",
        );
        fs::write(dir.join("commands/b.toml"), duplicate_order).expect("rewrite");
        let error = load_command_catalog(&dir).expect_err("duplicate topic order should fail");
        assert!(
            error
                .to_string()
                .contains("duplicate command training topic order")
        );

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn blank_canonical_fields_and_answers_are_rejected() {
        let cases = [
            (
                "command = \"grep foo\"",
                "command = \"   \"",
                "empty command",
            ),
            ("summary = \"Search\"", "summary = \" \"", "empty summary"),
            (
                "prompt = \"Search\"",
                "prompt = \"\"",
                "empty dictation prompt",
            ),
            (
                "answers = [\"grep foo\"]",
                "answers = []",
                "no dictation answers",
            ),
            (
                "answers = [\"grep foo\"]",
                "answers = [\"grep foo\", \"  \"]",
                "empty dictation answer",
            ),
        ];

        for (needle, replacement, expected) in cases {
            let dir = temp_data_dir();
            let fixture = command_fixture("grep-basic", "grep foo").replace(needle, replacement);
            fs::write(dir.join("commands/search.toml"), fixture).expect("write");
            let error = load_command_catalog(&dir).expect_err("blank canonical field should fail");
            assert!(
                error.to_string().contains(expected),
                "expected {expected:?}, got {error:#}"
            );
            fs::remove_dir_all(dir).expect("cleanup");
        }
    }

    #[test]
    fn missing_dir_returns_empty() {
        let dir = Path::new("/tmp/nonexistent-cmdtyper-dir-99999");
        let catalog = load_command_catalog(dir).expect("should not error on missing dir");
        assert!(catalog.commands.is_empty());
        assert!(catalog.topics.is_empty());
    }

    #[test]
    fn v02_optional_fields_have_defaults_and_legacy_files_have_no_topic() {
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

        let catalog = load_command_catalog(&dir).expect("should load");
        assert!(catalog.topics.is_empty());
        assert_eq!(catalog.commands[0].display.as_deref(), Some("ls -la"));
        assert_eq!(catalog.commands[0].summary_short.as_deref(), Some("list"));
        assert_eq!(
            catalog.commands[0].simulated_output.as_deref(),
            Some("total 0")
        );
        assert_eq!(catalog.commands[0].output_annotations.len(), 1);
        assert_eq!(catalog.commands[0].display_text(), "ls -la");
        assert_eq!(catalog.commands[0].short_summary(), "list");

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn filters_work() {
        let dir = temp_data_dir();
        fs::write(
            dir.join("commands/search.toml"),
            command_fixture("grep-basic", "grep foo"),
        )
        .expect("write");
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
        fs::write(dir.join("commands/archive.toml"), fixture2).expect("write");

        let commands = load_commands(&dir).expect("load");
        assert_eq!(load_by_difficulty(&commands, Difficulty::Beginner).len(), 1);
        assert_eq!(load_by_category(&commands, Category::Search).len(), 1);
        assert!(load_by_category(&commands, Category::FileOps).is_empty());

        fs::remove_dir_all(dir).expect("cleanup");
    }
}

/// Optional terminal dialect prompts, e.g. SQL statements entered inside psql.
pub fn load_command_prompts(
    data_dir: &Path,
    commands: &[Command],
) -> Result<HashMap<String, String>> {
    #[derive(serde::Deserialize)]
    struct Contexts {
        prompts: HashMap<String, String>,
    }
    let path = data_dir.join("command_contexts.toml");
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let contexts: Contexts = toml::from_str(&fs::read_to_string(path)?)?;
    for (id, prompt) in &contexts.prompts {
        if !commands.iter().any(|c| &c.id == id)
            || prompt.trim().is_empty()
            || prompt.chars().any(char::is_control)
        {
            bail!("invalid command prompt context {id}");
        }
    }
    Ok(contexts.prompts)
}
