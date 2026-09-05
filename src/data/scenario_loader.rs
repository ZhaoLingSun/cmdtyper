//! Data-driven, simulated operations cases and independently persisted progress.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use crate::data::models::{Command, Difficulty};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub title: String,
    pub category: String,
    pub difficulty: Difficulty,
    pub background: String,
    pub objective: String,
    pub environment: String,
    pub recap: String,
    #[serde(default)]
    pub sources: Vec<String>,
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioStep {
    pub id: String,
    pub title: String,
    pub instruction: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub command_id: Option<String>,
    #[serde(default = "default_prompt")]
    pub prompt: String,
    pub output: String,
    pub explanation: String,
    #[serde(default)]
    pub decision: Option<ScenarioDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDecision {
    pub question: String,
    pub choices: Vec<ScenarioChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioChoice {
    pub text: String,
    pub correct: bool,
    pub feedback: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioPhase {
    #[default]
    Intro,
    Typing,
    Evidence,
    Decision,
    Recap,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CaseProgress {
    pub step_index: usize,
    pub phase: ScenarioPhase,
    pub completed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ScenarioState {
    pub selected_index: usize,
    pub active_id: Option<String>,
    pub step_index: usize,
    pub phase: ScenarioPhase,
    pub choice_index: usize,
    pub feedback: Option<String>,
    pub scroll: usize,
    pub saved: BTreeMap<String, CaseProgress>,
}

impl ScenarioState {
    pub fn checkpoint(&mut self) {
        if let Some(id) = &self.active_id {
            self.saved.insert(
                id.clone(),
                CaseProgress {
                    step_index: self.step_index,
                    phase: self.phase,
                    completed: self.phase == ScenarioPhase::Recap,
                },
            );
        }
    }

    pub fn start(&mut self, scenario: &Scenario, restart: bool) {
        let saved = if restart {
            CaseProgress::default()
        } else {
            self.saved.get(&scenario.id).cloned().unwrap_or_default()
        };
        self.active_id = Some(scenario.id.clone());
        self.step_index = saved.step_index.min(scenario.steps.len().saturating_sub(1));
        self.phase = if saved.completed {
            ScenarioPhase::Recap
        } else {
            saved.phase
        };
        self.feedback = None;
        self.choice_index = 0;
        self.scroll = 0;
        self.checkpoint();
    }

    /// A correct choice advances. An incorrect choice returns to the evidence;
    /// there is no way to silently bypass a diagnostic checkpoint.
    pub fn answer(&mut self, scenario: &Scenario, choice_index: usize) -> bool {
        let choice = scenario
            .steps
            .get(self.step_index)
            .and_then(|step| step.decision.as_ref())
            .and_then(|decision| decision.choices.get(choice_index));
        let Some(choice) = choice else {
            return false;
        };
        self.feedback = Some(choice.feedback.clone());
        if choice.correct {
            self.advance(scenario);
        } else {
            self.phase = ScenarioPhase::Evidence;
            self.scroll = 0;
        }
        self.checkpoint();
        choice.correct
    }

    pub fn advance(&mut self, scenario: &Scenario) {
        self.choice_index = 0;
        self.scroll = 0;
        if self.step_index + 1 < scenario.steps.len() {
            self.step_index += 1;
            self.phase = ScenarioPhase::Typing;
        } else {
            self.phase = ScenarioPhase::Recap;
        }
        self.checkpoint();
    }
}

/// The argument is the scenarios directory, not the overall content directory.
pub fn load_scenarios(directory: &Path) -> Result<Vec<Scenario>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(directory)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.retain(|path| path.extension().is_some_and(|ext| ext == "toml"));
    paths.sort();
    let mut scenarios = Vec::new();
    let mut ids = BTreeSet::new();
    for path in paths {
        let content = fs::read_to_string(&path)?;
        let scenario: Scenario = toml::from_str(&content)
            .with_context(|| format!("invalid scenario {}", path.display()))?;
        validate_scenario(&scenario).with_context(|| path.display().to_string())?;
        ensure!(
            ids.insert(scenario.id.clone()),
            "duplicate scenario ID {}",
            scenario.id
        );
        scenarios.push(scenario);
    }
    Ok(scenarios)
}

pub fn validate_scenario(scenario: &Scenario) -> Result<()> {
    ensure!(!scenario.id.trim().is_empty(), "missing scenario ID");
    ensure!(!scenario.steps.is_empty(), "scenario has no steps");
    let mut step_ids = BTreeSet::new();
    for step in &scenario.steps {
        ensure!(step_ids.insert(&step.id), "duplicate step ID {}", step.id);
        ensure!(
            !step.command.trim().is_empty()
                || step
                    .command_id
                    .as_deref()
                    .is_some_and(|id| !id.trim().is_empty()),
            "empty command and reference in {}",
            step.id
        );
        ensure!(
            !step.command.contains(['\n', '\r']),
            "commands must be individually submitted in {}",
            step.id
        );
        ensure!(
            !step.instruction.trim().is_empty() && !step.explanation.trim().is_empty(),
            "missing teaching in {}",
            step.id
        );
        if let Some(decision) = &step.decision {
            ensure!(decision.choices.len() >= 2, "decision needs alternatives");
            ensure!(
                decision.choices.iter().filter(|c| c.correct).count() == 1,
                "decision needs exactly one correct answer"
            );
            ensure!(
                decision
                    .choices
                    .iter()
                    .all(|c| !c.feedback.trim().is_empty()),
                "decision feedback is required"
            );
        }
    }
    Ok(())
}

pub fn load_state(base_dir: &Path) -> ScenarioState {
    fs::read(base_dir.join("scenario_progress.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

pub fn save_state(base_dir: &Path, state: &ScenarioState) -> Result<()> {
    fs::create_dir_all(base_dir)?;
    let path = base_dir.join("scenario_progress.json");
    let temporary = base_dir.join("scenario_progress.json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(state)?)?;
    fs::rename(temporary, path)?;
    Ok(())
}

/// Resolve shared command references after the canonical catalog is loaded.
pub fn hydrate_scenarios(scenarios: &mut [Scenario], commands: &[Command]) -> Result<()> {
    let by_id = commands
        .iter()
        .map(|command| (command.id.as_str(), command.command.as_str()))
        .collect::<BTreeMap<_, _>>();
    for scenario in scenarios {
        for step in &mut scenario.steps {
            if let Some(id) = &step.command_id {
                let text = by_id.get(id.as_str()).with_context(|| {
                    format!("missing command {id} in {}:{}", scenario.id, step.id)
                })?;
                ensure!(
                    step.command.is_empty() || step.command == *text,
                    "command reference disagrees with embedded text in {}:{}",
                    scenario.id,
                    step.id
                );
                step.command = (*text).to_string();
            }
            ensure!(
                !step.command.trim().is_empty(),
                "unresolved command in {}:{}",
                scenario.id,
                step.id
            );
        }
        validate_scenario(scenario)?;
    }
    Ok(())
}

fn default_prompt() -> String {
    "$ ".to_string()
}
