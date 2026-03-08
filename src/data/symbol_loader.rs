use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::data::models::SymbolTopic;

/// Load all symbol topics from `data_dir/symbols/*.toml`.
pub fn load_symbol_topics(data_dir: &Path) -> Result<Vec<SymbolTopic>> {
    let symbols_dir = data_dir.join("symbols");
    let mut all_topics = Vec::new();

    if !symbols_dir.exists() {
        return Ok(all_topics);
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(&symbols_dir)
        .with_context(|| {
            format!(
                "failed to read symbols directory {}",
                symbols_dir.display()
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
        let topic: SymbolTopic = toml::from_str(&contents)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        all_topics.push(topic);
    }

    Ok(all_topics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::Difficulty;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_data_dir() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cmdtyper-symbol-test-{suffix}"));
        fs::create_dir_all(dir.join("symbols")).expect("temp dir should be created");
        dir
    }

    #[test]
    fn load_symbol_topics_from_toml() {
        let dir = temp_data_dir();
        let fixture = r#"
[meta]
id = "pipe_redirect"
topic = "管道与重定向"
description = "学习管道和重定向操作符"
difficulty = "basic"
icon = "🔀"

[[symbols]]
id = "pipe"
char_repr = "|"
name = "管道"
summary = "将前一个命令的输出作为后一个命令的输入"
explanation = "管道操作符 | 用于连接多个命令。"

[[symbols.examples]]
command = "cat file | grep error"
explanation = "在文件中搜索 error"

[[symbols.examples]]
command = "ls -la | sort -k5 -n"
explanation = "按文件大小排序"
display = "ls -la | sort -k5 -n"
simulated_output = "total 8\n-rw-r--r-- 1 user user 100 file.txt"

[[exercises]]
prompt = "将 ls 的输出通过管道传给 grep 搜索 txt"
answers = ["ls | grep txt", "ls | grep 'txt'"]
"#;

        fs::write(dir.join("symbols/pipe_redirect.toml"), fixture).expect("write");

        let topics = load_symbol_topics(&dir).expect("should load");
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].meta.id, "pipe_redirect");
        assert_eq!(topics[0].meta.difficulty, Difficulty::Basic);
        assert_eq!(topics[0].meta.icon.as_deref(), Some("🔀"));
        assert_eq!(topics[0].symbols.len(), 1);
        assert_eq!(topics[0].symbols[0].examples.len(), 2);
        assert_eq!(topics[0].exercises.len(), 1);
        assert_eq!(topics[0].exercises[0].answers.len(), 2);

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn missing_dir_returns_empty() {
        let dir = Path::new("/tmp/nonexistent-cmdtyper-symbols-99999");
        let topics = load_symbol_topics(dir).expect("should not error");
        assert!(topics.is_empty());
    }

    #[test]
    fn empty_exercises_default() {
        let dir = temp_data_dir();
        let fixture = r#"
[meta]
id = "wildcards"
topic = "通配符"
description = "通配符"
difficulty = "beginner"

[[symbols]]
id = "star"
char_repr = "*"
name = "星号通配符"
summary = "匹配任意字符"
explanation = "匹配零个或多个任意字符。"

[[symbols.examples]]
command = "ls *.txt"
explanation = "列出所有 txt 文件"
"#;

        fs::write(dir.join("symbols/wildcards.toml"), fixture).expect("write");

        let topics = load_symbol_topics(&dir).expect("load");
        assert_eq!(topics.len(), 1);
        assert!(topics[0].exercises.is_empty());
        assert!(topics[0].meta.icon.is_none());

        fs::remove_dir_all(dir).expect("cleanup");
    }
}
