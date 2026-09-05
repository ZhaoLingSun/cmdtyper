use std::fs;
use std::path::{Path, PathBuf};

fn rust_sources(directory: &Path) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    for entry in fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()))
    {
        let path = entry
            .expect("source directory entry should be readable")
            .path();
        if path.is_dir() {
            sources.extend(rust_sources(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(path);
        }
    }
    sources.sort();
    sources
}

fn runtime_portion(source: &str) -> &str {
    source.split("#[cfg(test)]").next().unwrap_or(source)
}

fn without_comments(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut cleaned = String::with_capacity(source.len());
    let mut index = 0;
    let mut block_depth = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut raw_hashes = None;

    while index < bytes.len() {
        if block_depth > 0 {
            if bytes[index..].starts_with(b"/*") {
                block_depth += 1;
                cleaned.push_str("  ");
                index += 2;
            } else if bytes[index..].starts_with(b"*/") {
                block_depth -= 1;
                cleaned.push_str("  ");
                index += 2;
            } else {
                cleaned.push(if bytes[index] == b'\n' { '\n' } else { ' ' });
                index += 1;
            }
            continue;
        }

        if let Some(hash_count) = raw_hashes {
            if bytes[index] == b'"'
                && bytes
                    .get(index + 1..index + 1 + hash_count)
                    .is_some_and(|hashes| hashes.iter().all(|byte| *byte == b'#'))
            {
                cleaned.push('"');
                cleaned.extend(std::iter::repeat_n('#', hash_count));
                index += hash_count + 1;
                raw_hashes = None;
            } else {
                cleaned.push(bytes[index] as char);
                index += 1;
            }
            continue;
        }

        if in_string {
            cleaned.push(bytes[index] as char);
            if escaped {
                escaped = false;
            } else if bytes[index] == b'\\' {
                escaped = true;
            } else if bytes[index] == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if bytes[index..].starts_with(b"//") {
            while index < bytes.len() && bytes[index] != b'\n' {
                cleaned.push(' ');
                index += 1;
            }
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            block_depth = 1;
            cleaned.push_str("  ");
            index += 2;
            continue;
        }
        if bytes[index] == b'"' {
            in_string = true;
            cleaned.push('"');
            index += 1;
            continue;
        }
        if bytes[index] == b'r' {
            let mut cursor = index + 1;
            while bytes.get(cursor) == Some(&b'#') {
                cursor += 1;
            }
            if bytes.get(cursor) == Some(&b'"') {
                let hash_count = cursor - index - 1;
                cleaned.push_str(&source[index..=cursor]);
                index = cursor + 1;
                raw_hashes = Some(hash_count);
                continue;
            }
        }

        cleaned.push(bytes[index] as char);
        index += 1;
    }

    cleaned
}

fn compact_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn runtime_rust_has_no_shell_or_process_execution_path() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let forbidden_compact = [
        "std::process::",
        "process::Command",
        "process::{Command",
        "Command::new(",
        ".spawn(",
        "tokio::process",
        "async_process::",
        "subprocess::",
        "duct::cmd(",
        "nix::unistd::fork(",
        "libc::fork(",
        "posix_spawn(",
        "execv(",
        "execve(",
        "execvp(",
        "/bin/sh",
        "/bin/bash",
        "/usr/bin/sh",
        "/usr/bin/bash",
    ];
    let forbidden_shell_phrases = ["sh -c", "bash -c"];
    let mut violations = Vec::new();

    for path in rust_sources(&source_root) {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let runtime = without_comments(runtime_portion(&source));
        let compact = compact_whitespace(&runtime);

        for signature in forbidden_compact {
            if compact.contains(signature) {
                violations.push(format!("{} contains {signature:?}", path.display()));
            }
        }
        for phrase in forbidden_shell_phrases {
            if runtime.contains(phrase) {
                violations.push(format!("{} contains {phrase:?}", path.display()));
            }
        }
        for shell in ["sh", "bash"] {
            if runtime.contains(&format!("\"{shell}\"")) && runtime.contains("\"-c\"") {
                violations.push(format!(
                    "{} contains {shell:?} with a separate \"-c\" argument",
                    path.display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "cmdtyper must never execute shell commands or spawn processes:\n{}",
        violations.join("\n")
    );
}
