//! Integration test: verify that tokens text concatenation == command
//! for every command in every command TOML file.

use std::fs;
use std::path::Path;

use cmdtyper::data::models::CommandFile;

#[test]
fn tokens_concat_equals_command() {
    let dir = Path::new("data").join("commands");
    let mut total_commands = 0;
    let mut errors = Vec::new();

    for entry in fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", dir.display()))
    {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Cannot read {}: {e}", path.display()));
            let cf: CommandFile = toml::from_str(&content)
                .unwrap_or_else(|e| panic!("Parse error in {}: {e}", path.display()));

            for cmd in &cf.commands {
                let concatenated: String =
                    cmd.tokens.iter().map(|t| t.text.as_str()).collect();
                if concatenated != cmd.command {
                    errors.push(format!(
                        "  {} (id={}): tokens concat = {:?}, command = {:?}",
                        path.display(),
                        cmd.id,
                        concatenated,
                        cmd.command,
                    ));
                }
                total_commands += 1;
            }
        }
    }

    if !errors.is_empty() {
        panic!(
            "Token concatenation mismatches found ({}/{} commands):\n{}",
            errors.len(),
            total_commands,
            errors.join("\n")
        );
    }

    println!(
        "All {total_commands} commands have consistent tokens/command fields"
    );
}
