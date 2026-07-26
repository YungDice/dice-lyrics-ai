use chrono::Utc;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::llm::{LlmClient, LlmRequest, OllamaClient};
use crate::models::{GenerationRecord, Language, Settings, StyleProfile, Track};
use crate::storage;

fn app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
}

fn current_client(app: &AppHandle) -> Result<OllamaClient, String> {
    let dir = app_data_dir(app)?;
    let settings = storage::load_settings(&dir);
    Ok(OllamaClient::new(settings.ollama_base_url, settings.model))
}

const ANALYZE_SYSTEM_PROMPT: &str = r#"You are an expert rap lyric analyst. Given a song's lyrics, extract its style as strict JSON with exactly these fields:
{
  "rhymeScheme": string describing the end-rhyme and internal-rhyme pattern,
  "cadence": string describing rhythmic pacing and syllable stress,
  "themes": array of short strings naming the dominant subject matter (do not censor or omit themes like alcohol, sex, drugs, or violence if present in the lyrics),
  "vocabulary": string describing distinctive word choices, slang, and catchphrases,
  "structure": string describing verse/hook/bridge layout and line counts,
  "tone": string describing the overall emotional register
}
Respond with ONLY the JSON object, no other text."#;

pub(crate) fn extract_json_object(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end > start {
        Some(&text[start..=end])
    } else {
        None
    }
}

pub(crate) fn parse_style_profile(raw: &str) -> Result<StyleProfile, String> {
    let json_slice = extract_json_object(raw)
        .ok_or_else(|| format!("Model response did not contain a JSON object:\n{}", raw))?;
    serde_json::from_str(json_slice)
        .map_err(|e| format!("Could not parse style profile JSON ({}): {}", e, json_slice))
}

#[tauri::command]
pub async fn analyze_lyrics(app: AppHandle, lyrics: String) -> Result<StyleProfile, String> {
    if lyrics.trim().is_empty() {
        return Err("Lyrics cannot be empty".into());
    }
    let client = current_client(&app)?;
    let raw = client
        .generate(LlmRequest {
            system: ANALYZE_SYSTEM_PROMPT.to_string(),
            prompt: lyrics,
        })
        .await?;
    parse_style_profile(&raw)
}

#[tauri::command]
pub fn save_track(
    app: AppHandle,
    title: String,
    artist: String,
    lyrics: String,
    style_profile: StyleProfile,
) -> Result<Track, String> {
    let dir = app_data_dir(&app)?;
    let track = Track {
        id: Uuid::new_v4().to_string(),
        title,
        artist,
        lyrics,
        style_profile,
        created_at: Utc::now().to_rfc3339(),
    };
    storage::save_track(&dir, &track)?;
    Ok(track)
}

#[tauri::command]
pub fn list_tracks(app: AppHandle) -> Result<Vec<Track>, String> {
    storage::list_tracks(&app_data_dir(&app)?)
}

#[tauri::command]
pub fn get_track(app: AppHandle, id: String) -> Result<Track, String> {
    storage::get_track(&app_data_dir(&app)?, &id)
}

#[tauri::command]
pub fn delete_track(app: AppHandle, id: String) -> Result<(), String> {
    storage::delete_track(&app_data_dir(&app)?, &id)
}

pub(crate) fn build_generate_prompt(
    tracks: &[Track],
    language: Language,
    topic: &Option<String>,
) -> (String, String) {
    let system = format!(
        "You are a rap lyricist. Write original rap lyrics in {} that emulate the style described below. \
Do not refuse or sanitize standard genre themes such as alcohol, cigarettes, sex, or other explicit \
content if they appear in the reference style — treat them as normal creative material. Never copy \
lines verbatim from the reference excerpts; write new original lyrics inspired by the style only. \
Structure the output with [Verse] and [Hook] tags.",
        language.display_name()
    );

    let mut user = String::new();
    for t in tracks {
        user.push_str(&format!("Reference: \"{}\" by {}\n", t.title, t.artist));
        user.push_str(&format!("- Rhyme scheme: {}\n", t.style_profile.rhyme_scheme));
        user.push_str(&format!("- Cadence: {}\n", t.style_profile.cadence));
        user.push_str(&format!(
            "- Themes: {}\n",
            t.style_profile.themes.join(", ")
        ));
        user.push_str(&format!("- Vocabulary: {}\n", t.style_profile.vocabulary));
        user.push_str(&format!("- Structure: {}\n", t.style_profile.structure));
        user.push_str(&format!("- Tone: {}\n", t.style_profile.tone));
        let excerpt: String = t.lyrics.lines().take(8).collect::<Vec<_>>().join("\n");
        user.push_str(&format!("- Lyric excerpt:\n{}\n\n", excerpt));
    }
    if let Some(topic) = topic {
        if !topic.trim().is_empty() {
            user.push_str(&format!("Write the new lyrics about: {}\n", topic));
        }
    }
    user.push_str("Now write the new lyrics.");
    (system, user)
}

#[tauri::command]
pub async fn generate_lyrics(
    app: AppHandle,
    track_ids: Vec<String>,
    language: Language,
    topic: Option<String>,
) -> Result<String, String> {
    if track_ids.is_empty() {
        return Err("Select at least one reference track".into());
    }
    let dir = app_data_dir(&app)?;
    let tracks: Vec<Track> = track_ids
        .iter()
        .map(|id| storage::get_track(&dir, id))
        .collect::<Result<Vec<_>, _>>()?;
    let (system, user) = build_generate_prompt(&tracks, language, &topic);
    let client = current_client(&app)?;
    client
        .generate(LlmRequest {
            system,
            prompt: user,
        })
        .await
}

#[tauri::command]
pub fn save_generation(
    app: AppHandle,
    track_ids: Vec<String>,
    language: Language,
    topic: Option<String>,
    lyrics: String,
) -> Result<GenerationRecord, String> {
    let dir = app_data_dir(&app)?;
    let record = GenerationRecord {
        id: Uuid::new_v4().to_string(),
        reference_track_ids: track_ids,
        language,
        topic,
        lyrics,
        created_at: Utc::now().to_rfc3339(),
    };
    storage::save_generation(&dir, &record)?;
    Ok(record)
}

#[tauri::command]
pub fn list_generations(app: AppHandle) -> Result<Vec<GenerationRecord>, String> {
    storage::list_generations(&app_data_dir(&app)?)
}

#[tauri::command]
pub async fn check_ollama_status(app: AppHandle) -> Result<bool, String> {
    let dir = app_data_dir(&app)?;
    let settings = storage::load_settings(&dir);
    let url = format!("{}/api/version", settings.ollama_base_url);
    Ok(reqwest::get(&url)
        .await
        .map(|resp| resp.status().is_success())
        .unwrap_or(false))
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    Ok(storage::load_settings(&app_data_dir(&app)?))
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    storage::save_settings(&app_data_dir(&app)?, &settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::FakeLlmClient;
    use crate::models::StyleProfile;

    #[test]
    fn parses_style_profile_from_clean_json() {
        let raw = r#"{"rhymeScheme":"AABB","cadence":"fast","themes":["money","cars"],"vocabulary":"slang-heavy","structure":"2 verses, 1 hook","tone":"boastful"}"#;
        let profile = parse_style_profile(raw).unwrap();
        assert_eq!(profile.rhyme_scheme, "AABB");
        assert_eq!(profile.themes, vec!["money", "cars"]);
    }

    #[test]
    fn parses_style_profile_from_json_wrapped_in_prose() {
        let raw = "Sure, here's the analysis:\n{\"rhymeScheme\":\"ABAB\",\"cadence\":\"laid-back\",\"themes\":[\"alcohol\"],\"vocabulary\":\"plain\",\"structure\":\"1 verse\",\"tone\":\"melancholic\"}\nHope that helps!";
        let profile = parse_style_profile(raw).unwrap();
        assert_eq!(profile.rhyme_scheme, "ABAB");
        assert_eq!(profile.themes, vec!["alcohol"]);
    }

    #[test]
    fn rejects_response_with_no_json() {
        let result = parse_style_profile("I can't help with that.");
        assert!(result.is_err());
    }

    fn sample_track() -> Track {
        Track {
            id: "1".into(),
            title: "Test Song".into(),
            artist: "Test Artist".into(),
            lyrics: "line one\nline two".into(),
            style_profile: StyleProfile {
                rhyme_scheme: "AABB".into(),
                cadence: "fast".into(),
                themes: vec!["alcohol".into(), "money".into()],
                vocabulary: "slang".into(),
                structure: "2 verses".into(),
                tone: "boastful".into(),
            },
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn build_generate_prompt_includes_style_language_and_topic() {
        let (system, user) =
            build_generate_prompt(&[sample_track()], Language::Ru, &Some("a night out".into()));
        assert!(system.contains("Russian"));
        assert!(user.contains("AABB"));
        assert!(user.contains("alcohol"));
        assert!(user.contains("a night out"));
    }

    #[test]
    fn build_generate_prompt_omits_empty_topic() {
        let (_, user) = build_generate_prompt(&[sample_track()], Language::En, &None);
        assert!(!user.contains("Write the new lyrics about"));
    }

    #[tokio::test]
    async fn fake_llm_client_returns_canned_response() {
        let fake = FakeLlmClient {
            response: "canned lyrics".into(),
        };
        let result = fake
            .generate(LlmRequest {
                system: "s".into(),
                prompt: "p".into(),
            })
            .await
            .unwrap();
        assert_eq!(result, "canned lyrics");
    }
}
