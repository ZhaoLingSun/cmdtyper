use std::collections::HashMap;

use anyhow::{Result, bail};

use crate::data::models::{
    Command, CommandCatalog, CommandLesson, ExampleTokenDetail, Exercise, LessonExample,
    SymbolExample, SymbolTopic, SystemCommand, SystemTopic,
};

/// Hydrate canonical command references across learning content and validate
/// stable IDs in the namespace where each record type is addressed.
pub fn hydrate_and_validate_content(
    catalog: &CommandCatalog,
    lessons: &mut [CommandLesson],
    symbol_topics: &mut [SymbolTopic],
    system_topics: &mut [SystemTopic],
) -> Result<()> {
    let commands = build_command_lookup(&catalog.commands)?;
    let mut lesson_identities = HashMap::new();

    for (lesson_index, lesson) in lessons.iter_mut().enumerate() {
        let lesson_context = format!("lesson {:?}", lesson.meta.command);
        validate_required_scoped_id(
            &mut lesson_identities,
            &lesson.meta.command,
            &format!("lesson at index {lesson_index}"),
            "lesson identity",
        )?;

        let mut example_ids = HashMap::new();
        for (example_index, example) in lesson.examples.iter_mut().enumerate() {
            let context = format!("{lesson_context} example {example_index}");
            validate_optional_scoped_id(
                &mut example_ids,
                example.id.as_deref(),
                &context,
                "lesson example ID",
            )?;
            hydrate_lesson_example(example, &commands, &context)?;
            if example.level == 0 {
                bail!("{context} must have a positive level");
            }
        }
    }

    let mut symbol_topic_ids = HashMap::new();
    for (topic_index, topic) in symbol_topics.iter_mut().enumerate() {
        let topic_context = format!("symbol topic {:?}", topic.meta.id);
        validate_required_scoped_id(
            &mut symbol_topic_ids,
            &topic.meta.id,
            &format!("symbol topic at index {topic_index}"),
            "symbol topic ID",
        )?;

        let mut symbol_entry_ids = HashMap::new();
        let mut record_ids = HashMap::new();
        for (symbol_index, symbol) in topic.symbols.iter_mut().enumerate() {
            let symbol_context = format!("{topic_context}, symbol {:?}", symbol.id);
            validate_required_scoped_id(
                &mut symbol_entry_ids,
                &symbol.id,
                &format!("{topic_context}, symbol at index {symbol_index}"),
                "symbol entry ID",
            )?;
            for (example_index, example) in symbol.examples.iter_mut().enumerate() {
                let context = format!("{symbol_context}, example {example_index}");
                validate_optional_scoped_id(
                    &mut record_ids,
                    example.id.as_deref(),
                    &context,
                    "symbol example/exercise ID",
                )?;
                hydrate_symbol_example(example, &commands, &context)?;
            }
        }
        for (exercise_index, exercise) in topic.exercises.iter_mut().enumerate() {
            let context = format!("{topic_context}, exercise {exercise_index}");
            validate_reference_backed_id(
                exercise.id.as_deref(),
                exercise.command_id.as_deref(),
                &context,
                "exercise",
            )?;
            validate_optional_scoped_id(
                &mut record_ids,
                exercise.id.as_deref(),
                &context,
                "symbol example/exercise ID",
            )?;
            hydrate_exercise(exercise, &commands, &context)?;
        }
    }

    let mut system_topic_ids = HashMap::new();
    for (topic_index, topic) in system_topics.iter_mut().enumerate() {
        let topic_context = format!("system topic {:?}", topic.meta.id);
        validate_required_scoped_id(
            &mut system_topic_ids,
            &topic.meta.id,
            &format!("system topic at index {topic_index}"),
            "system topic ID",
        )?;

        let mut section_ids = HashMap::new();
        for (section_index, section) in topic.sections.iter_mut().enumerate() {
            let section_context = format!("{topic_context}, section {:?}", section.id);
            validate_required_scoped_id(
                &mut section_ids,
                &section.id,
                &format!("{topic_context}, section at index {section_index}"),
                "system section ID",
            )?;

            let mut command_ids = HashMap::new();
            for (command_index, system_command) in section.commands.iter_mut().enumerate() {
                let context = format!("{section_context}, command {command_index}");
                validate_reference_backed_id(
                    system_command.id.as_deref(),
                    system_command.command_id.as_deref(),
                    &context,
                    "system command",
                )?;
                validate_optional_scoped_id(
                    &mut command_ids,
                    system_command.id.as_deref(),
                    &context,
                    "system command ID",
                )?;
                hydrate_system_command(system_command, &commands, &context)?;
            }
        }
    }

    Ok(())
}

pub(crate) fn validate_canonical_commands(commands: &[Command]) -> Result<()> {
    let mut command_ids = HashMap::new();
    for (index, command) in commands.iter().enumerate() {
        let context = format!("canonical command {:?} at index {index}", command.id);
        if command.id.trim().is_empty() {
            bail!("canonical command ID must not be empty");
        }
        if let Some(previous_context) = command_ids.insert(command.id.as_str(), context.clone()) {
            bail!(
                "duplicate canonical command ID {:?} in {previous_context} and {context}",
                command.id
            );
        }
        if command.command.trim().is_empty() {
            bail!("{context} has an empty command");
        }
        if command.summary.trim().is_empty() {
            bail!("{context} has an empty summary");
        }
        if command.dictation.prompt.trim().is_empty() {
            bail!("{context} has an empty dictation prompt");
        }
        if command.dictation.answers.is_empty() {
            bail!("{context} has no dictation answers");
        }
        if command
            .dictation
            .answers
            .iter()
            .any(|answer| answer.trim().is_empty())
        {
            bail!("{context} has an empty dictation answer");
        }
    }
    Ok(())
}

fn build_command_lookup(commands: &[Command]) -> Result<HashMap<&str, &Command>> {
    validate_canonical_commands(commands)?;
    Ok(commands
        .iter()
        .map(|command| (command.id.as_str(), command))
        .collect())
}

fn validate_required_scoped_id(
    ids: &mut HashMap<String, String>,
    id: &str,
    context: &str,
    namespace: &str,
) -> Result<()> {
    if id.trim().is_empty() {
        bail!("{context} has an empty {namespace}");
    }
    if let Some(previous_context) = ids.insert(id.to_string(), context.to_string()) {
        bail!(
            "duplicate {namespace} {:?} in {previous_context} and {context}",
            id
        );
    }
    Ok(())
}

fn validate_optional_scoped_id(
    ids: &mut HashMap<String, String>,
    id: Option<&str>,
    context: &str,
    namespace: &str,
) -> Result<()> {
    let Some(id) = id else {
        return Ok(());
    };
    validate_required_scoped_id(ids, id, context, namespace)
}

fn validate_reference_backed_id(
    id: Option<&str>,
    command_id: Option<&str>,
    context: &str,
    record_type: &str,
) -> Result<()> {
    if command_id.is_some() && id.is_none() {
        bail!("{context} is a reference-backed {record_type} and must provide a stable ID");
    }
    if command_id.is_some_and(|value| value.trim().is_empty()) {
        bail!("{context} has an empty canonical command reference");
    }
    if id.is_some_and(|value| value.trim().is_empty()) {
        bail!("{context} has an empty stable ID");
    }
    Ok(())
}

fn referenced_command<'a>(
    command_id: Option<&str>,
    commands: &HashMap<&'a str, &'a Command>,
    context: &str,
) -> Result<Option<&'a Command>> {
    let Some(command_id) = command_id else {
        return Ok(None);
    };
    if command_id.trim().is_empty() {
        bail!("{context} has an empty canonical command reference");
    }
    commands
        .get(command_id)
        .copied()
        .map(Some)
        .ok_or_else(|| anyhow::anyhow!("{context} references unknown command ID {command_id:?}"))
}

fn hydrate_lesson_example(
    example: &mut LessonExample,
    commands: &HashMap<&str, &Command>,
    context: &str,
) -> Result<()> {
    let Some(command) = referenced_command(example.command_id.as_deref(), commands, context)?
    else {
        require_inline_string(&example.command, "command", context)?;
        require_inline_string(&example.summary, "summary", context)?;
        return Ok(());
    };

    merge_string(&mut example.command, &command.command, "command", context)?;
    merge_string(&mut example.summary, &command.summary, "summary", context)?;
    merge_option(
        &mut example.display,
        command.display.as_deref(),
        "display",
        context,
    )?;
    merge_option(
        &mut example.simulated_output,
        command.simulated_output.as_deref(),
        "simulated_output",
        context,
    )?;
    merge_vec(
        &mut example.output_annotations,
        &command.output_annotations,
        "output_annotations",
        context,
    )?;

    let canonical_token_details: Vec<ExampleTokenDetail> = command
        .tokens
        .iter()
        .map(|token| ExampleTokenDetail {
            token: token.text.clone(),
            explanation: token.desc.clone(),
            kind: token.kind,
        })
        .collect();
    merge_vec(
        &mut example.token_details,
        &canonical_token_details,
        "token_details",
        context,
    )?;
    Ok(())
}

fn hydrate_symbol_example(
    example: &mut SymbolExample,
    commands: &HashMap<&str, &Command>,
    context: &str,
) -> Result<()> {
    let Some(command) = referenced_command(example.command_id.as_deref(), commands, context)?
    else {
        require_inline_string(&example.command, "command", context)?;
        require_inline_string(&example.explanation, "explanation", context)?;
        return Ok(());
    };

    merge_string(&mut example.command, &command.command, "command", context)?;
    merge_string(
        &mut example.explanation,
        &command.summary,
        "explanation",
        context,
    )?;
    merge_option(
        &mut example.display,
        command.display.as_deref(),
        "display",
        context,
    )?;
    merge_option(
        &mut example.simulated_output,
        command.simulated_output.as_deref(),
        "simulated_output",
        context,
    )?;
    Ok(())
}

fn hydrate_exercise(
    exercise: &mut Exercise,
    commands: &HashMap<&str, &Command>,
    context: &str,
) -> Result<()> {
    let Some(command) = referenced_command(exercise.command_id.as_deref(), commands, context)?
    else {
        require_inline_string(&exercise.prompt, "prompt", context)?;
        if exercise.answers.is_empty() {
            bail!("{context} must provide answers or a canonical command reference");
        }
        if exercise
            .answers
            .iter()
            .any(|answer| answer.trim().is_empty())
        {
            bail!("{context} has an empty answer");
        }
        return Ok(());
    };

    merge_string(
        &mut exercise.prompt,
        &command.dictation.prompt,
        "prompt",
        context,
    )?;
    merge_vec(
        &mut exercise.answers,
        &command.dictation.answers,
        "answers",
        context,
    )?;
    merge_option(
        &mut exercise.command,
        Some(&command.command),
        "command",
        context,
    )?;
    merge_option(
        &mut exercise.simulated_output,
        command.simulated_output.as_deref(),
        "simulated_output",
        context,
    )?;
    Ok(())
}

fn hydrate_system_command(
    system_command: &mut SystemCommand,
    commands: &HashMap<&str, &Command>,
    context: &str,
) -> Result<()> {
    let Some(command) =
        referenced_command(system_command.command_id.as_deref(), commands, context)?
    else {
        require_inline_string(&system_command.command, "command", context)?;
        require_inline_string(&system_command.summary, "summary", context)?;
        return Ok(());
    };

    merge_string(
        &mut system_command.command,
        &command.command,
        "command",
        context,
    )?;
    merge_string(
        &mut system_command.summary,
        &command.summary,
        "summary",
        context,
    )?;
    merge_option(
        &mut system_command.simulated_output,
        command.simulated_output.as_deref(),
        "simulated_output",
        context,
    )?;
    Ok(())
}

fn require_inline_string(value: &str, field: &str, context: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{context} must provide {field:?} or a canonical command reference");
    }
    Ok(())
}

fn merge_string(target: &mut String, canonical: &str, field: &str, context: &str) -> Result<()> {
    if target.is_empty() {
        target.push_str(canonical);
    } else if target != canonical {
        bail!(
            "{context} field {field:?} disagrees with canonical command: embedded {:?}, canonical {:?}",
            target,
            canonical
        );
    }
    Ok(())
}

fn merge_option(
    target: &mut Option<String>,
    canonical: Option<&str>,
    field: &str,
    context: &str,
) -> Result<()> {
    let Some(canonical) = canonical else {
        return Ok(());
    };
    match target {
        Some(embedded) if embedded != canonical => bail!(
            "{context} field {field:?} disagrees with canonical command: embedded {:?}, canonical {:?}",
            embedded,
            canonical
        ),
        Some(_) => Ok(()),
        None => {
            *target = Some(canonical.to_string());
            Ok(())
        }
    }
}

fn merge_vec<T>(target: &mut Vec<T>, canonical: &[T], field: &str, context: &str) -> Result<()>
where
    T: Clone + PartialEq + std::fmt::Debug,
{
    if target.is_empty() {
        target.extend_from_slice(canonical);
    } else if target != canonical {
        bail!(
            "{context} field {field:?} disagrees with canonical command: embedded {:?}, canonical {:?}",
            target,
            canonical
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::{
        DictationData, SymbolEntry, SymbolTopicMeta, SystemSection, SystemTopicMeta, Token,
        TokenKind,
    };

    fn canonical_command() -> Command {
        Command {
            id: "grep-basic".to_string(),
            command: "grep error app.log".to_string(),
            summary: "搜索错误日志".to_string(),
            tokens: vec![
                Token {
                    text: "grep".to_string(),
                    desc: "搜索文本".to_string(),
                    kind: Some(TokenKind::Command),
                },
                Token {
                    text: " error app.log".to_string(),
                    desc: "模式与文件".to_string(),
                    kind: None,
                },
            ],
            dictation: DictationData {
                prompt: "在 app.log 中搜索 error".to_string(),
                answers: vec!["grep error app.log".to_string()],
            },
            display: Some("grep error app.log".to_string()),
            simulated_output: Some("error: failed".to_string()),
            ..Command::default()
        }
    }

    fn catalog() -> CommandCatalog {
        CommandCatalog {
            commands: vec![canonical_command()],
            topics: Vec::new(),
        }
    }

    fn lesson(command: &str, examples: &str) -> CommandLesson {
        toml::from_str(&format!(
            r#"
[meta]
command = "{command}"
category = "search"
difficulty = "basic"
[overview]
summary = "grep"
explanation = "grep"
[syntax]
basic = "grep"
{examples}
"#
        ))
        .expect("lesson should parse")
    }

    fn symbol_topic(id: &str, body: &str) -> SymbolTopic {
        toml::from_str(&format!(
            r#"
[meta]
id = "{id}"
topic = "Symbols"
description = "Symbols"
difficulty = "basic"
{body}
"#
        ))
        .expect("symbol topic should parse")
    }

    fn system_topic(id: &str, body: &str) -> SystemTopic {
        toml::from_str(&format!(
            r#"
[meta]
id = "{id}"
topic = "System"
description = "System"
difficulty = "basic"
{body}
"#
        ))
        .expect("system topic should parse")
    }

    #[test]
    fn reference_only_records_are_hydrated_from_canonical_command() {
        let mut lessons = vec![lesson(
            "grep",
            r#"
[[examples]]
id = "lesson-grep"
command_id = "grep-basic"
level = 1
"#,
        )];
        let mut symbols = vec![symbol_topic(
            "pipe",
            r#"
[[symbols]]
id = "pipe-symbol"
char_repr = "|"
name = "Pipe"
summary = "Pipe"
explanation = "Pipe"
[[symbols.examples]]
id = "symbol-grep"
command_id = "grep-basic"
[[exercises]]
id = "exercise-grep"
command_id = "grep-basic"
"#,
        )];
        let mut systems = vec![system_topic(
            "logs",
            r#"
[[sections]]
id = "search"
title = "Search"
description = "Search"
[[sections.commands]]
id = "system-grep"
command_id = "grep-basic"
"#,
        )];

        hydrate_and_validate_content(&catalog(), &mut lessons, &mut symbols, &mut systems)
            .expect("references should hydrate");

        let lesson = &lessons[0].examples[0];
        assert_eq!(lesson.command, "grep error app.log");
        assert_eq!(lesson.summary, "搜索错误日志");
        assert_eq!(lesson.token_details.len(), 2);
        let symbol = &symbols[0].symbols[0].examples[0];
        assert_eq!(symbol.command, "grep error app.log");
        assert_eq!(symbol.explanation, "搜索错误日志");
        let exercise = &symbols[0].exercises[0];
        assert_eq!(exercise.prompt, "在 app.log 中搜索 error");
        assert_eq!(exercise.answers, ["grep error app.log"]);
        let system = &systems[0].sections[0].commands[0];
        assert_eq!(system.command, "grep error app.log");
        assert_eq!(system.summary, "搜索错误日志");
    }

    #[test]
    fn reference_backed_exercise_and_system_command_require_stable_ids() {
        let mut symbols = vec![symbol_topic(
            "pipe",
            r#"
[[symbols]]
id = "pipe-symbol"
char_repr = "|"
name = "Pipe"
summary = "Pipe"
explanation = "Pipe"
examples = []
[[exercises]]
command_id = "grep-basic"
"#,
        )];
        let error = hydrate_and_validate_content(&catalog(), &mut [], &mut symbols, &mut [])
            .expect_err("reference-backed exercise without ID should fail");
        assert!(error.to_string().contains("reference-backed exercise"));
        assert!(error.to_string().contains("stable ID"));

        let mut systems = vec![system_topic(
            "logs",
            r#"
[[sections]]
id = "search"
title = "Search"
description = "Search"
[[sections.commands]]
command_id = "grep-basic"
"#,
        )];
        let error = hydrate_and_validate_content(&catalog(), &mut [], &mut [], &mut systems)
            .expect_err("reference-backed system command without ID should fail");
        assert!(
            error
                .to_string()
                .contains("reference-backed system command")
        );
        assert!(error.to_string().contains("stable ID"));
    }

    #[test]
    fn lesson_example_level_must_be_positive_after_hydration() {
        for examples in [
            r#"
[[examples]]
command = "grep error app.log"
summary = "搜索错误日志"
"#,
            r#"
[[examples]]
id = "lesson-grep"
command_id = "grep-basic"
"#,
        ] {
            let mut lessons = vec![lesson("grep", examples)];
            let error = hydrate_and_validate_content(&catalog(), &mut lessons, &mut [], &mut [])
                .expect_err("zero lesson level should fail");
            assert!(error.to_string().contains("must have a positive level"));
        }
    }

    #[test]
    fn embedded_canonical_fields_must_agree_exactly() {
        let mut lessons = vec![lesson(
            "grep",
            r#"
[[examples]]
id = "lesson-grep"
command_id = "grep-basic"
level = 1
command = "grep warning app.log"
summary = "搜索错误日志"
"#,
        )];
        let error = hydrate_and_validate_content(&catalog(), &mut lessons, &mut [], &mut [])
            .expect_err("disagreement should fail");
        assert!(error.to_string().contains("field \"command\" disagrees"));

        let mut symbols = vec![symbol_topic(
            "pipe",
            r#"
[[symbols]]
id = "pipe-symbol"
char_repr = "|"
name = "Pipe"
summary = "Pipe"
explanation = "Pipe"
[[symbols.examples]]
command_id = "grep-basic"
explanation = "different"
"#,
        )];
        let error = hydrate_and_validate_content(&catalog(), &mut [], &mut symbols, &mut [])
            .expect_err("embedded explanation disagreement should fail");
        assert!(
            error
                .to_string()
                .contains("field \"explanation\" disagrees")
        );
    }

    #[test]
    fn canonical_commands_reject_blank_fields_and_answers_before_hydration() {
        type CommandMutation = Box<dyn Fn(&mut Command)>;

        let mutations: Vec<CommandMutation> = vec![
            Box::new(|command| command.command = " ".to_string()),
            Box::new(|command| command.summary.clear()),
            Box::new(|command| command.dictation.prompt = "\t".to_string()),
            Box::new(|command| command.dictation.answers.clear()),
            Box::new(|command| command.dictation.answers.push(" ".to_string())),
        ];
        let expected = [
            "empty command",
            "empty summary",
            "empty dictation prompt",
            "no dictation answers",
            "empty dictation answer",
        ];

        for (mutate, expected) in mutations.into_iter().zip(expected) {
            let mut command = canonical_command();
            mutate(&mut command);
            let invalid_catalog = CommandCatalog {
                commands: vec![command],
                topics: Vec::new(),
            };
            let error = hydrate_and_validate_content(&invalid_catalog, &mut [], &mut [], &mut [])
                .expect_err("blank canonical content should fail");
            assert!(error.to_string().contains(expected));
        }
    }

    #[test]
    fn duplicate_ids_are_rejected_in_their_own_namespaces() {
        let valid_example = r#"
[[examples]]
id = "example-one"
level = 1
command = "ls"
summary = "list"
"#;
        let mut lessons = vec![lesson("grep", valid_example), lesson("grep", valid_example)];
        let error = hydrate_and_validate_content(&catalog(), &mut lessons, &mut [], &mut [])
            .expect_err("duplicate lesson identity should fail");
        assert!(error.to_string().contains("duplicate lesson identity"));

        let duplicate_examples = r#"
[[examples]]
id = "same"
level = 1
command = "ls"
summary = "list"
[[examples]]
id = "same"
level = 2
command = "pwd"
summary = "cwd"
"#;
        let mut lessons = vec![lesson("grep", duplicate_examples)];
        let error = hydrate_and_validate_content(&catalog(), &mut lessons, &mut [], &mut [])
            .expect_err("duplicate lesson example ID should fail");
        assert!(error.to_string().contains("duplicate lesson example ID"));

        let symbol_body = r#"
[[symbols]]
id = "same"
char_repr = "*"
name = "Star"
summary = "Star"
explanation = "Star"
examples = []
[[symbols]]
id = "same"
char_repr = "?"
name = "Question"
summary = "Question"
explanation = "Question"
examples = []
"#;
        let mut symbols = vec![symbol_topic("symbols", symbol_body)];
        let error = hydrate_and_validate_content(&catalog(), &mut [], &mut symbols, &mut [])
            .expect_err("duplicate symbol entry ID should fail");
        assert!(error.to_string().contains("duplicate symbol entry ID"));

        let symbol_records = r#"
[[symbols]]
id = "star"
char_repr = "*"
name = "Star"
summary = "Star"
explanation = "Star"
[[symbols.examples]]
id = "same"
command = "ls"
explanation = "list"
[[exercises]]
id = "same"
prompt = "list"
answers = ["ls"]
"#;
        let mut symbols = vec![symbol_topic("symbols", symbol_records)];
        let error = hydrate_and_validate_content(&catalog(), &mut [], &mut symbols, &mut [])
            .expect_err("duplicate symbol record ID should fail");
        assert!(
            error
                .to_string()
                .contains("duplicate symbol example/exercise ID")
        );

        let single_symbol = r#"
[[symbols]]
id = "entry"
char_repr = "*"
name = "Star"
summary = "Star"
explanation = "Star"
examples = []
"#;
        let mut symbols = vec![
            symbol_topic("symbols", single_symbol),
            symbol_topic("symbols", single_symbol),
        ];
        let error = hydrate_and_validate_content(&catalog(), &mut [], &mut symbols, &mut [])
            .expect_err("duplicate symbol topic ID should fail");
        assert!(error.to_string().contains("duplicate symbol topic ID"));

        let duplicate_sections = r#"
[[sections]]
id = "same"
title = "One"
description = "One"
[[sections]]
id = "same"
title = "Two"
description = "Two"
"#;
        let mut systems = vec![system_topic("system", duplicate_sections)];
        let error = hydrate_and_validate_content(&catalog(), &mut [], &mut [], &mut systems)
            .expect_err("duplicate system section ID should fail");
        assert!(error.to_string().contains("duplicate system section ID"));

        let duplicate_commands = r#"
[[sections]]
id = "commands"
title = "Commands"
description = "Commands"
[[sections.commands]]
id = "same"
command = "ls"
summary = "list"
[[sections.commands]]
id = "same"
command = "pwd"
summary = "cwd"
"#;
        let mut systems = vec![system_topic("system", duplicate_commands)];
        let error = hydrate_and_validate_content(&catalog(), &mut [], &mut [], &mut systems)
            .expect_err("duplicate system command ID should fail");
        assert!(error.to_string().contains("duplicate system command ID"));

        let single_section = r#"
[[sections]]
id = "one"
title = "One"
description = "One"
"#;
        let mut systems = vec![
            system_topic("system", single_section),
            system_topic("system", single_section),
        ];
        let error = hydrate_and_validate_content(&catalog(), &mut [], &mut [], &mut systems)
            .expect_err("duplicate system topic ID should fail");
        assert!(error.to_string().contains("duplicate system topic ID"));
    }

    #[test]
    fn duplicate_ids_in_different_scopes_remain_valid() {
        let examples = r#"
[[examples]]
id = "shared"
level = 1
command = "ls"
summary = "list"
"#;
        let mut lessons = vec![lesson("grep", examples), lesson("find", examples)];
        let symbol_body = r#"
[[symbols]]
id = "entry"
char_repr = "*"
name = "Star"
summary = "Star"
explanation = "Star"
[[symbols.examples]]
id = "shared"
command = "ls"
explanation = "list"
"#;
        let mut symbols = vec![
            symbol_topic("one", symbol_body),
            symbol_topic("two", symbol_body),
        ];
        let system_body = r#"
[[sections]]
id = "one"
title = "One"
description = "One"
[[sections.commands]]
id = "shared"
command = "ls"
summary = "list"
[[sections]]
id = "two"
title = "Two"
description = "Two"
[[sections.commands]]
id = "shared"
command = "pwd"
summary = "cwd"
"#;
        let mut systems = vec![system_topic("system", system_body)];

        hydrate_and_validate_content(&catalog(), &mut lessons, &mut symbols, &mut systems)
            .expect("IDs should only be compared inside their namespace");
    }

    #[test]
    fn duplicate_canonical_command_id_is_rejected() {
        let command = canonical_command();
        let duplicate_catalog = CommandCatalog {
            commands: vec![command.clone(), command],
            topics: Vec::new(),
        };

        let error = hydrate_and_validate_content(&duplicate_catalog, &mut [], &mut [], &mut [])
            .expect_err("duplicate canonical ID should fail");
        assert!(error.to_string().contains("duplicate canonical command ID"));
    }

    #[test]
    fn legacy_inline_records_may_remain_id_less() {
        let mut lessons = vec![lesson(
            "grep",
            r#"
[[examples]]
level = 1
command = "ls"
summary = "list"
"#,
        )];
        let mut symbols = vec![SymbolTopic {
            meta: SymbolTopicMeta {
                id: "symbols".to_string(),
                topic: "Symbols".to_string(),
                description: "Symbols".to_string(),
                difficulty: Default::default(),
                icon: None,
            },
            symbols: vec![SymbolEntry {
                id: "star".to_string(),
                char_repr: "*".to_string(),
                name: "Star".to_string(),
                summary: "Star".to_string(),
                explanation: "Star".to_string(),
                examples: vec![SymbolExample {
                    id: None,
                    command_id: None,
                    command: "ls".to_string(),
                    explanation: "list".to_string(),
                    display: None,
                    simulated_output: None,
                    deep_explanation: None,
                }],
            }],
            exercises: vec![Exercise {
                id: None,
                command_id: None,
                prompt: "list".to_string(),
                answers: vec!["ls".to_string()],
                kind: None,
                command: None,
                simulated_output: None,
            }],
        }];
        let mut systems = vec![SystemTopic {
            meta: SystemTopicMeta {
                id: "system".to_string(),
                topic: "System".to_string(),
                description: "System".to_string(),
                difficulty: Default::default(),
                icon: None,
            },
            overview: None,
            sections: vec![SystemSection {
                id: "commands".to_string(),
                title: "Commands".to_string(),
                description: "Commands".to_string(),
                commands: vec![SystemCommand {
                    id: None,
                    command_id: None,
                    command: "ls".to_string(),
                    summary: "list".to_string(),
                    simulated_output: None,
                    deep_explanation: None,
                }],
                config_files: Vec::new(),
            }],
        }];

        hydrate_and_validate_content(&catalog(), &mut lessons, &mut symbols, &mut systems)
            .expect("legacy inline records should not require stable IDs");
    }
}
