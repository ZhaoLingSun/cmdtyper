use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::data::models::CommandLesson;

/// Load all command lessons from `data_dir/lessons/*.toml`.
pub fn load_lessons(data_dir: &Path) -> Result<Vec<CommandLesson>> {
    let lessons_dir = data_dir.join("lessons");
    let mut all_lessons = Vec::new();

    if !lessons_dir.exists() {
        return Ok(all_lessons);
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(&lessons_dir)
        .with_context(|| format!("failed to read lessons directory {}", lessons_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect();

    entries.sort();

    for path in entries {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let lesson: CommandLesson = toml::from_str(&contents)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        all_lessons.push(lesson);
    }

    Ok(all_lessons)
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
        let dir = std::env::temp_dir().join(format!("cmdtyper-lesson-test-{suffix}"));
        fs::create_dir_all(dir.join("lessons")).expect("temp dir should be created");
        dir
    }

    #[test]
    fn load_lessons_from_toml() {
        let dir = temp_data_dir();
        let fixture = r#"
[meta]
command = "ls"
full_name = "list"
category = "file_ops"
difficulty = "beginner"
importance = "core"

[overview]
summary = "列出目录内容"
explanation = "ls 命令用于列出文件和目录。"

[syntax]
basic = "ls [选项] [目录...]"
[[syntax.parts]]
name = "目录"
desc = "要列出的目录路径"

[[options]]
flag = "-l"
name = "长格式"
example = "ls -l"

[[examples]]
level = 1
command = "ls"
summary = "列出当前目录"

[[gotchas]]
title = "隐藏文件"
content = "默认不显示以 . 开头的文件"
"#;

        fs::write(dir.join("lessons/ls.toml"), fixture).expect("write");

        let lessons = load_lessons(&dir).expect("should load");
        assert_eq!(lessons.len(), 1);
        assert_eq!(lessons[0].meta.command, "ls");
        assert_eq!(lessons[0].meta.full_name.as_deref(), Some("list"));
        assert_eq!(lessons[0].syntax.parts.len(), 1);
        assert_eq!(lessons[0].options.len(), 1);
        assert_eq!(lessons[0].examples.len(), 1);
        assert_eq!(lessons[0].gotchas.len(), 1);

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn missing_dir_returns_empty() {
        let dir = Path::new("/tmp/nonexistent-cmdtyper-lessons-99999");
        let lessons = load_lessons(dir).expect("should not error");
        assert!(lessons.is_empty());
    }

    #[test]
    fn optional_fields_default() {
        let dir = temp_data_dir();
        let fixture = r#"
[meta]
command = "cat"
category = "file_ops"
difficulty = "beginner"

[overview]
summary = "显示文件内容"
explanation = "cat 命令。"

[syntax]
basic = "cat [文件...]"

[[examples]]
level = 1
command = "cat file.txt"
summary = "显示文件"
"#;

        fs::write(dir.join("lessons/cat.toml"), fixture).expect("write");

        let lessons = load_lessons(&dir).expect("load");
        assert_eq!(lessons.len(), 1);
        assert!(lessons[0].meta.full_name.is_none());
        assert!(lessons[0].options.is_empty());
        assert!(lessons[0].gotchas.is_empty());
        assert!(lessons[0].syntax.parts.is_empty());

        fs::remove_dir_all(dir).expect("cleanup");
    }
}
