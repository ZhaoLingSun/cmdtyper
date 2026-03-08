use std::fs;
use std::path::Path;

#[test]
fn parse_all_real_toml_files() {
    let dir = Path::new("data/commands");
    assert!(dir.exists(), "data/commands directory should exist");

    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "toml"))
        .collect();
    entries.sort();

    for path in &entries {
        let content = fs::read_to_string(path).unwrap();
        let result: Result<toml::Value, _> = toml::from_str(&content);
        match &result {
            Ok(_) => eprintln!("✅ {}", path.display()),
            Err(e) => eprintln!("❌ {} : {}", path.display(), e),
        }
        assert!(result.is_ok(), "Failed raw parse: {}", path.display());
    }
}
