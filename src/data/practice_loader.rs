use crate::data::models::Command;
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};

#[derive(Debug, Clone, Deserialize)]
pub struct PracticeGroup {
    pub id: String,
    pub source_kind: String,
    pub source_id: String,
    pub unit_id: String,
    pub title: String,
    pub teaching_command_id: String,
    pub exercise_command_ids: Vec<String>,
}
#[derive(Deserialize)]
struct PracticeFile {
    groups: Vec<PracticeGroup>,
}

pub fn load_practice_groups(data_dir: &Path, commands: &[Command]) -> Result<Vec<PracticeGroup>> {
    let directory = data_dir.join("practice");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(&directory)?
        .map(|p| p.map(|p| p.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.sort();
    let mut groups = Vec::new();
    let known: HashSet<_> = commands.iter().map(|c| c.id.as_str()).collect();
    let mut seen = HashSet::new();
    for path in paths
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
    {
        let file: PracticeFile = toml::from_str(&fs::read_to_string(&path)?)
            .with_context(|| format!("practice {}", path.display()))?;
        for group in file.groups {
            ensure!(
                !group.id.is_empty() && seen.insert(group.id.clone()),
                "duplicate/empty practice group {}",
                group.id
            );
            ensure!(
                ["lesson", "symbol", "system"].contains(&group.source_kind.as_str()),
                "invalid source kind: {}",
                group.id
            );
            ensure!(
                !group.source_id.is_empty() && !group.unit_id.is_empty() && !group.title.is_empty(),
                "missing practice metadata: {}",
                group.id
            );
            ensure!(
                known.contains(group.teaching_command_id.as_str()),
                "unknown teaching command: {}",
                group.id
            );
            ensure!(
                group.exercise_command_ids.len() == 3,
                "{} needs exactly three exercises",
                group.id
            );
            let mut unique = HashSet::new();
            for id in &group.exercise_command_ids {
                ensure!(
                    known.contains(id.as_str())
                        && unique.insert(id)
                        && id != &group.teaching_command_id,
                    "invalid/duplicate exercise {id} in {}",
                    group.id
                );
            }
            groups.push(group);
        }
    }
    Ok(groups)
}

pub fn validate_practice_sources(
    groups: &[PracticeGroup],
    lessons: &[crate::data::models::CommandLesson],
    symbols: &[crate::data::models::SymbolTopic],
    systems: &[crate::data::models::SystemTopic],
) -> Result<()> {
    for group in groups {
        let exists = match group.source_kind.as_str() {
            "lesson" => lessons.iter().any(|l| {
                l.meta.command == group.source_id
                    && l.examples.iter().any(|e| {
                        e.level == 1
                            && e.command_id.as_deref() == Some(group.teaching_command_id.as_str())
                    })
            }),
            "symbol" => symbols.iter().any(|t| {
                t.meta.id == group.source_id && t.symbols.iter().any(|e| e.id == group.unit_id)
            }),
            "system" => systems.iter().any(|t| {
                t.meta.id == group.source_id && t.sections.iter().any(|e| e.id == group.unit_id)
            }),
            _ => false,
        };
        ensure!(
            exists,
            "practice {} references unknown foundation {}:{}",
            group.id,
            group.source_id,
            group.unit_id
        );
    }
    Ok(())
}
