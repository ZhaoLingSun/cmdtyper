use serde::Deserialize;
use toml;

#[derive(Debug, Clone, Deserialize)]
pub struct ReviewItemData {
    pub command: String,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub difficulty: String,
    #[serde(default)]
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub answer: String,
    #[serde(default)]
    pub command_id: String,
}

#[derive(Debug, Deserialize)]
struct ReviewToml {
    exercises: Vec<ReviewItemData>,
}

pub fn data_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
}

pub fn load_command_review(topic_name: &str) -> Vec<ReviewItemData> {
    let file_path = data_dir().join("reviews").join(format!("{}.toml", topic_name));

    if !file_path.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    match toml::from_str::<ReviewToml>(&content) {
        Ok(parsed) => parsed.exercises,
        Err(e) => {
            eprintln!("Failed to parse {}: {}", file_path.display(), e);
            Vec::new()
        }
    }
}

pub fn load_symbol_review(topic_name: &str) -> Vec<ReviewItemData> {
    load_command_review(topic_name)
}
