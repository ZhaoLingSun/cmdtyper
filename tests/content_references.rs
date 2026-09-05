use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use cmdtyper::data::catalog::hydrate_and_validate_content;
use cmdtyper::data::command_loader::load_command_catalog;
use cmdtyper::data::lesson_loader::load_lessons;
use cmdtyper::data::models::{Command, CommandCatalog, CommandLesson, SymbolTopic, SystemTopic};
use cmdtyper::data::symbol_loader::load_symbol_topics;
use cmdtyper::data::system_loader::load_system_topics;

const DATA_DIR: &str = "data";
const EXPECTED_TOPICS: [(u16, &str); 16] = [
    (1, "help_rescue"),
    (2, "apt_workflow"),
    (3, "tar_zip"),
    (4, "fileops_safety"),
    (5, "redirect_pipe"),
    (6, "env_shell"),
    (7, "find_grep"),
    (8, "disk_space"),
    (9, "process_port"),
    (10, "ssh_remote"),
    (11, "systemd_cron"),
    (12, "vim_survival"),
    (13, "users_permission"),
    (14, "text_toolkit"),
    (15, "zh_locale"),
    (16, "terminal_session_recovery"),
];

struct LoadedContent {
    catalog: CommandCatalog,
    lessons: Vec<CommandLesson>,
    symbols: Vec<SymbolTopic>,
    systems: Vec<SystemTopic>,
}

fn toml_files(directory: &Path) -> Vec<PathBuf> {
    let mut files: Vec<_> = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()))
        .map(|entry| entry.expect("directory entry should be readable").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "toml")
        })
        .collect();
    files.sort();
    files
}

fn load_and_hydrate() -> LoadedContent {
    let data_dir = Path::new(DATA_DIR);
    let catalog = load_command_catalog(data_dir).expect("real command catalog should load");
    let mut lessons = load_lessons(data_dir).expect("real lessons should load");
    let mut symbols = load_symbol_topics(data_dir).expect("real symbol topics should load");
    let mut systems = load_system_topics(data_dir).expect("real system topics should load");

    hydrate_and_validate_content(&catalog, &mut lessons, &mut symbols, &mut systems)
        .expect("all real content references and ID namespaces should validate");

    LoadedContent {
        catalog,
        lessons,
        symbols,
        systems,
    }
}

fn command_lookup(commands: &[Command]) -> HashMap<&str, &Command> {
    commands
        .iter()
        .map(|command| (command.id.as_str(), command))
        .collect()
}

fn assert_nonempty(value: &str, context: &str) {
    assert!(!value.trim().is_empty(), "{context} must not be empty");
}

fn assert_unique<'a>(ids: impl IntoIterator<Item = &'a str>, namespace: &str) {
    let mut seen = HashSet::new();
    for id in ids {
        assert_nonempty(id, namespace);
        assert!(
            seen.insert(id),
            "duplicate ID {id:?} in {namespace} namespace"
        );
    }
}

#[test]
fn exact_content_inventory_loads_and_hydrates() {
    let content = load_and_hydrate();

    assert_eq!(
        toml_files(&Path::new(DATA_DIR).join("commands")).len(),
        35,
        "unexpected command file inventory"
    );
    assert_eq!(
        content.catalog.commands.len(),
        554,
        "unexpected canonical command inventory"
    );
    assert_eq!(
        content.catalog.topics.len(),
        16,
        "unexpected ordered topic inventory"
    );

    let ordered_topics: Vec<_> = content
        .catalog
        .topics
        .iter()
        .map(|topic| (topic.order, topic.id.as_str()))
        .collect();
    assert_eq!(
        ordered_topics, EXPECTED_TOPICS,
        "command topic IDs or order changed"
    );

    let canonical_ids: HashSet<_> = content
        .catalog
        .commands
        .iter()
        .map(|command| command.id.as_str())
        .collect();
    let mut topic_command_ids = HashSet::new();
    for topic in &content.catalog.topics {
        for command_id in &topic.command_ids {
            assert!(
                canonical_ids.contains(command_id.as_str()),
                "topic {:?} references unknown command ID {:?}",
                topic.id,
                command_id
            );
            assert!(
                topic_command_ids.insert(command_id.as_str()),
                "command ID {command_id:?} is mapped to more than one topic"
            );
        }
    }
    assert_eq!(
        topic_command_ids.len(),
        283,
        "unexpected uniquely topic-mapped command inventory"
    );

    assert_eq!(content.lessons.len(), 75, "unexpected lesson inventory");
    assert_eq!(
        content.symbols.len(),
        8,
        "unexpected symbol topic inventory"
    );
    assert_eq!(
        content
            .symbols
            .iter()
            .map(|topic| topic.exercises.len())
            .sum::<usize>(),
        120,
        "unexpected symbol exercise inventory"
    );
    assert_eq!(
        content.systems.len(),
        11,
        "unexpected system topic inventory"
    );
    assert_eq!(
        content
            .systems
            .iter()
            .map(|topic| topic.sections.len())
            .sum::<usize>(),
        52,
        "unexpected system section inventory"
    );
}

#[test]
fn every_topic_command_is_referenced_once_by_v03_lessons() {
    let content = load_and_hydrate();
    let canonical_ids: HashSet<_> = content
        .catalog
        .commands
        .iter()
        .map(|command| command.id.as_str())
        .collect();
    let topic_command_ids: HashSet<_> = content
        .catalog
        .topics
        .iter()
        .flat_map(|topic| topic.command_ids.iter().map(String::as_str))
        .collect();

    let v03_files: Vec<_> = toml_files(&Path::new(DATA_DIR).join("lessons"))
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("v03_"))
        })
        .collect();
    assert_eq!(v03_files.len(), 44, "unexpected v03 lesson file inventory");

    let mut reference_counts: HashMap<String, usize> = HashMap::new();
    for path in v03_files {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let lesson: CommandLesson = toml::from_str(&source)
            .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()));

        for (index, example) in lesson.examples.iter().enumerate() {
            let command_id = example.command_id.as_deref().unwrap_or_else(|| {
                panic!(
                    "{} example {index} must reference a canonical topic command",
                    path.display()
                )
            });
            assert!(
                canonical_ids.contains(command_id),
                "{} example {index} references unknown command ID {command_id:?}",
                path.display()
            );
            *reference_counts.entry(command_id.to_string()).or_default() += 1;
        }
    }

    assert_eq!(
        reference_counts.len(),
        283,
        "v03 lessons must reference all 283 topic commands"
    );
    assert_eq!(
        reference_counts
            .keys()
            .map(String::as_str)
            .collect::<HashSet<_>>(),
        topic_command_ids,
        "v03 lesson references must exactly match the topic command inventory"
    );
    for (command_id, count) in reference_counts {
        assert_eq!(
            count, 1,
            "topic command ID {command_id:?} must be referenced exactly once"
        );
    }
}

#[test]
fn reference_backed_records_hydrate_canonical_data_and_stable_ids() {
    let content = load_and_hydrate();
    let commands = command_lookup(&content.catalog.commands);

    for lesson in &content.lessons {
        for example in &lesson.examples {
            let Some(command_id) = example.command_id.as_deref() else {
                continue;
            };
            let canonical = commands
                .get(command_id)
                .unwrap_or_else(|| panic!("lesson references unknown command ID {command_id:?}"));
            assert_eq!(example.command, canonical.command);
            assert_eq!(example.summary, canonical.summary);
            assert_eq!(example.token_details.len(), canonical.tokens.len());
            assert!(
                example
                    .token_details
                    .iter()
                    .all(|token| !token.token.is_empty() && !token.explanation.trim().is_empty()),
                "lesson {:?} reference {command_id:?} did not hydrate canonical tokens",
                lesson.meta.command
            );
        }
    }

    for topic in &content.symbols {
        for symbol in &topic.symbols {
            for example in &symbol.examples {
                let Some(command_id) = example.command_id.as_deref() else {
                    continue;
                };
                let canonical = commands.get(command_id).unwrap_or_else(|| {
                    panic!("symbol example references unknown command ID {command_id:?}")
                });
                assert_eq!(example.command, canonical.command);
                assert_eq!(example.explanation, canonical.summary);
            }
        }

        for exercise in &topic.exercises {
            let Some(command_id) = exercise.command_id.as_deref() else {
                continue;
            };
            let stable_id = exercise.id.as_deref().unwrap_or_else(|| {
                panic!("reference-backed exercise for {command_id:?} needs a stable ID")
            });
            assert_nonempty(stable_id, "reference-backed exercise ID");

            let canonical = commands.get(command_id).unwrap_or_else(|| {
                panic!("symbol exercise references unknown command ID {command_id:?}")
            });
            assert_eq!(exercise.prompt, canonical.dictation.prompt);
            assert_eq!(exercise.answers, canonical.dictation.answers);
            assert_eq!(
                exercise.command.as_deref(),
                Some(canonical.command.as_str())
            );
        }
    }

    for topic in &content.systems {
        for section in &topic.sections {
            for command in &section.commands {
                let Some(command_id) = command.command_id.as_deref() else {
                    continue;
                };
                let stable_id = command.id.as_deref().unwrap_or_else(|| {
                    panic!("reference-backed system command for {command_id:?} needs a stable ID")
                });
                assert_nonempty(stable_id, "reference-backed system command ID");

                let canonical = commands.get(command_id).unwrap_or_else(|| {
                    panic!("system command references unknown command ID {command_id:?}")
                });
                assert_eq!(command.command, canonical.command);
                assert_eq!(command.summary, canonical.summary);
            }
        }
    }
}

#[test]
fn documented_content_id_namespaces_are_unique() {
    let content = load_and_hydrate();

    assert_unique(
        content
            .lessons
            .iter()
            .map(|lesson| lesson.meta.command.as_str()),
        "lesson meta.command (global)",
    );
    assert_unique(
        content.symbols.iter().map(|topic| topic.meta.id.as_str()),
        "symbol topic ID (global)",
    );
    assert_unique(
        content.systems.iter().map(|topic| topic.meta.id.as_str()),
        "system topic ID (global)",
    );

    for lesson in &content.lessons {
        assert_unique(
            lesson
                .examples
                .iter()
                .filter_map(|example| example.id.as_deref()),
            &format!("lesson {:?} example ID", lesson.meta.command),
        );
    }

    for topic in &content.symbols {
        assert_unique(
            topic.symbols.iter().map(|symbol| symbol.id.as_str()),
            &format!("symbol topic {:?} entry ID", topic.meta.id),
        );
        assert_unique(
            topic
                .symbols
                .iter()
                .flat_map(|symbol| symbol.examples.iter())
                .filter_map(|example| example.id.as_deref())
                .chain(
                    topic
                        .exercises
                        .iter()
                        .filter_map(|exercise| exercise.id.as_deref()),
                ),
            &format!("symbol topic {:?} example/exercise ID", topic.meta.id),
        );
    }

    for topic in &content.systems {
        assert_unique(
            topic.sections.iter().map(|section| section.id.as_str()),
            &format!("system topic {:?} section ID", topic.meta.id),
        );
        for section in &topic.sections {
            assert_unique(
                section
                    .commands
                    .iter()
                    .filter_map(|command| command.id.as_deref()),
                &format!(
                    "system topic {:?} section {:?} command ID",
                    topic.meta.id, section.id
                ),
            );
        }
    }
}
