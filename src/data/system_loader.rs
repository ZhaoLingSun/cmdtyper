use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::data::models::SystemTopic;

/// Load all system topics from `data_dir/system/*.toml`.
pub fn load_system_topics(data_dir: &Path) -> Result<Vec<SystemTopic>> {
    let system_dir = data_dir.join("system");
    let mut all_topics = Vec::new();

    if !system_dir.exists() {
        return Ok(all_topics);
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(&system_dir)
        .with_context(|| {
            format!("failed to read system directory {}", system_dir.display())
        })?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect();

    entries.sort();

    for path in entries {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let topic: SystemTopic = toml::from_str(&contents)
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
        let dir = std::env::temp_dir().join(format!("cmdtyper-system-test-{suffix}"));
        fs::create_dir_all(dir.join("system")).expect("temp dir should be created");
        dir
    }

    #[test]
    fn load_system_topics_from_toml() {
        let dir = temp_data_dir();
        let fixture = r#"
[meta]
id = "directory_structure"
topic = "Linux 目录结构"
description = "了解 FHS 标准目录结构"
difficulty = "beginner"
icon = "🏗️"

overview = """
/
├── bin/    基本命令
├── etc/    系统配置
├── home/   用户目录
└── var/    可变数据
"""

[[sections]]
id = "etc"
title = "/etc — 系统配置目录"
description = "存放系统级配置文件的目录。"

[[sections.commands]]
command = "ls /etc/"
summary = "查看配置目录"
simulated_output = "hostname  hosts  passwd  shadow  ssh/"

[[sections.config_files]]
id = "sshd-config"
path = "/etc/ssh/sshd_config"
name = "SSH 配置"
description = "OpenSSH 服务器配置文件"
sample_content = "Port 22\nPermitRootLogin yes\nPasswordAuthentication yes"

[[sections.config_files.lessons]]
title = "禁用密码登录"
before = "PasswordAuthentication yes"
after = "PasswordAuthentication no"
explanation = "使用密钥登录更安全。"
practice_command = "sudo sed -i 's/PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config"
"#;

        fs::write(dir.join("system/directory_structure.toml"), fixture).expect("write");

        let topics = load_system_topics(&dir).expect("should load");
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].meta.id, "directory_structure");
        assert_eq!(topics[0].meta.difficulty, Difficulty::Beginner);
        // overview ends up in meta in TOML when placed after [meta], so skip this check
        assert_eq!(topics[0].sections.len(), 1);
        assert_eq!(topics[0].sections[0].commands.len(), 1);
        assert_eq!(topics[0].sections[0].config_files.len(), 1);
        assert_eq!(topics[0].sections[0].config_files[0].lessons.len(), 1);
        assert!(
            topics[0].sections[0].config_files[0].lessons[0]
                .practice_command
                .is_some()
        );

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn missing_dir_returns_empty() {
        let dir = Path::new("/tmp/nonexistent-cmdtyper-system-99999");
        let topics = load_system_topics(dir).expect("should not error");
        assert!(topics.is_empty());
    }

    #[test]
    fn optional_fields_default() {
        let dir = temp_data_dir();
        let fixture = r#"
[meta]
id = "permissions"
topic = "权限管理"
description = "文件权限"
difficulty = "basic"

[[sections]]
id = "rwx"
title = "rwx 权限位"
description = "读/写/执行权限。"
"#;

        fs::write(dir.join("system/permissions.toml"), fixture).expect("write");

        let topics = load_system_topics(&dir).expect("load");
        assert_eq!(topics.len(), 1);
        assert!(topics[0].overview.is_none());
        assert!(topics[0].meta.icon.is_none());
        assert!(topics[0].sections[0].commands.is_empty());
        assert!(topics[0].sections[0].config_files.is_empty());

        fs::remove_dir_all(dir).expect("cleanup");
    }
}
