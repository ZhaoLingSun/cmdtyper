//! Integration test: verify all command IDs are globally unique
//! across every command TOML file.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use cmdtyper::data::models::CommandFile;

#[test]
fn all_command_ids_unique() {
    let dir = Path::new("data").join("commands");
    // Map from id -> file where it was first seen
    let mut seen: HashMap<String, String> = HashMap::new();
    let mut duplicates = Vec::new();

    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("Cannot read {}: {e}", dir.display()))
    {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));
            let cf: CommandFile = toml::from_str(&content)
                .unwrap_or_else(|e| panic!("Parse error in {}: {e}", path.display()));

            let filename = path.display().to_string();
            for cmd in &cf.commands {
                if let Some(prev_file) = seen.get(&cmd.id) {
                    duplicates.push(format!(
                        "  id={:?} in {} (first seen in {})",
                        cmd.id, filename, prev_file
                    ));
                } else {
                    seen.insert(cmd.id.clone(), filename.clone());
                }
            }
        }
    }

    if !duplicates.is_empty() {
        panic!("Duplicate command IDs found:\n{}", duplicates.join("\n"));
    }

    println!("All {} command IDs are globally unique", seen.len());
}
