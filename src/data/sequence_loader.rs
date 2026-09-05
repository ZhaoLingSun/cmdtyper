use crate::data::models::Command;
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};

/// A workflow keeps its legacy identity while referencing individual commands.
#[derive(Debug, Clone, Deserialize)]
pub struct CommandSequence {
    pub id: String,
    #[serde(default)]
    pub command_ids: Vec<String>,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub steps: Vec<SequenceStep>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SequenceStep {
    #[serde(default)]
    pub command_id: String,
    #[serde(default, skip_deserializing)]
    pub command: String,
    /// Some("") deliberately describes a silent command in this context.
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub explanation: String,
    #[serde(default)]
    pub prompt: Option<String>,
}

#[derive(Deserialize)]
struct SequenceFile {
    sequences: Vec<CommandSequence>,
}

pub fn load_sequences(data_dir: &Path, catalog: &[Command]) -> Result<Vec<CommandSequence>> {
    let directory = data_dir.join("sequences");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.sort();
    let mut result = Vec::new();
    let mut ids = HashSet::new();
    for path in paths
        .into_iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
    {
        let file: SequenceFile = toml::from_str(&fs::read_to_string(&path)?)
            .with_context(|| format!("sequence {}", path.display()))?;
        for mut sequence in file.sequences {
            ensure!(
                ids.insert(sequence.id.clone()),
                "duplicate sequence {}",
                sequence.id
            );
            let target = catalog
                .iter()
                .find(|command| command.id == sequence.id)
                .with_context(|| format!("unknown sequence {}", sequence.id))?;
            if !sequence.steps.is_empty() {
                let references: Vec<_> = sequence
                    .steps
                    .iter()
                    .map(|step| step.command_id.clone())
                    .collect();
                ensure!(
                    sequence.command_ids.is_empty() || sequence.command_ids == references,
                    "inconsistent workflow step references {}",
                    sequence.id
                );
                sequence.command_ids = references;
            }
            if !sequence.command_ids.is_empty() {
                if sequence.steps.is_empty() {
                    sequence.steps = sequence
                        .command_ids
                        .iter()
                        .map(|id| SequenceStep {
                            command_id: id.clone(),
                            ..SequenceStep::default()
                        })
                        .collect();
                }
                let mut commands = Vec::new();
                for step in &mut sequence.steps {
                    let command = catalog
                        .iter()
                        .find(|command| command.id == step.command_id)
                        .with_context(|| format!("unknown sequence step {}", step.command_id))?;
                    ensure!(
                        !command.command.contains('\n'),
                        "nested workflow step {}",
                        step.command_id
                    );
                    step.command = command.command.clone();
                    if step.output.is_none() {
                        step.output = command.simulated_output.clone();
                    }
                    if step.explanation.trim().is_empty() {
                        step.explanation = command.short_summary().to_owned();
                    }
                    commands.push(command.command.clone());
                }
                ensure!(
                    sequence.commands.is_empty() || sequence.commands == commands,
                    "inconsistent workflow strings {}",
                    sequence.id
                );
                sequence.commands = commands;
            } else {
                // Old string-only data remains readable without inventing command IDs.
                sequence.steps = sequence
                    .commands
                    .iter()
                    .map(|text| {
                        let reference = catalog.iter().find(|command| command.command == *text);
                        SequenceStep {
                            command_id: reference
                                .map(|command| command.id.clone())
                                .unwrap_or_default(),
                            command: text.clone(),
                            output: reference.and_then(|command| command.simulated_output.clone()),
                            explanation: reference
                                .map(|command| command.short_summary().to_owned())
                                .unwrap_or_else(|| format!("按当前流程的准备条件输入：{text}")),
                            prompt: None,
                        }
                    })
                    .collect();
            }
            ensure!(
                sequence.commands.len() >= 2
                    && sequence
                        .commands
                        .iter()
                        .all(|text| !text.trim().is_empty() && !text.contains('\n')),
                "invalid workflow {}",
                sequence.id
            );
            ensure!(
                sequence.commands.join("\n") == target.command,
                "workflow target disagrees {}",
                sequence.id
            );
            result.push(sequence);
        }
    }
    Ok(result)
}
