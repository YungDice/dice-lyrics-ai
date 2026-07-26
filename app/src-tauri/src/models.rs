use serde::{Deserialize, Serialize};

/// Four deep, evidence-quoting analysis sections plus short tags for the
/// library card. All fields default so tracks saved by older versions still
/// load (their deep sections will just be empty until re-analyzed).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleProfile {
    #[serde(default)]
    pub rhyme_and_flow: String,
    #[serde(default)]
    pub vocabulary_and_slang: String,
    #[serde(default)]
    pub themes_and_imagery: String,
    #[serde(default)]
    pub structure_and_delivery: String,
    #[serde(default)]
    pub themes: Vec<String>,
    #[serde(default)]
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
