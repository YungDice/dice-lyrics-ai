use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleProfile {
    pub rhyme_scheme: String,
    pub cadence: String,
    pub themes: Vec<String>,
    pub vocabulary: String,
    pub structure: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub lyrics: String,
    pub style_profile: StyleProfile,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Ru,
    De,
}

impl Language {
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::En => "English",
            Language::Ru => "Russian",
            Language::De => "German",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationRecord {
    pub id: String,
    pub reference_track_ids: Vec<String>,
    pub language: Language,
    pub topic: Option<String>,
    pub lyrics: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub ollama_base_url: String,
    pub model: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ollama_base_url: "http://localhost:11434".to_string(),
            model: "dolphin-mistral:7b".to_string(),
        }
    }
}
