use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    stage: String,
    stage_index: usize,
    total_stages: usize,
    kind: String, // "start" | "token" | "stage-done"
    text: String,
}

fn emit(app: &AppHandle, event: &str, payload: ProgressEvent) {
    let _ = app.emit(event, payload);
}

// ---------------------------------------------------------------------------
// Analysis: four exhaustive passes + one summary pass, all streamed.
// ---------------------------------------------------------------------------

const ANALYSIS_STAGES: [(&str, &str); 4] = [
    (
        "Rhyme & flow",
        r#"You are a world-class rap lyricist and battle-rap analyst. Analyze ONLY the rhyme craft and flow of the lyrics the user provides. Be exhaustive and concrete — every claim must quote actual words from the lyrics as evidence. Cover, as compact bullet points:
- End-rhyme scheme per section (AABB, ABAB, slant rhymes...), quoting the actual rhyming word pairs
- Internal rhymes: quote every internal-rhyme chain you find
- Multisyllabic rhymes: quote them and count the syllables
- Rhyme density: rate it low/medium/high and estimate rhymes per bar
- Flow: typical syllables per bar (estimate a range), where the stresses land, double-time or half-time passages, where pauses and breath points fall
- Signature flow tricks (rhyme stacking, delayed rhymes, off-beat landings)
Only rhyme and flow — do not discuss themes or vocabulary. Quote lyrics exactly as written; do not censor them."#,
    ),
    (
        "Vocabulary, slang & ad-libs",
        r#"You are a rap linguistics expert. Analyze ONLY the vocabulary and verbal signature of the lyrics the user provides. Be exhaustive and concrete — quote actual words from the lyrics as evidence. Cover, as compact bullet points:
- Slang dictionary: every slang term used, with its meaning in context
- Ad-libs and interjections (e.g. "Huh", "Yeah", "Skrrt"): list each one and where it lands
- Catchphrases or repeated signature words
- Profanity level and how it is deployed
- Borrowed words or language mixing, brand names, place names, artist references
- Overall register (street, playful, literary, aggressive...)
Only vocabulary — do not discuss rhyme schemes or song structure. Quote lyrics exactly as written; do not censor them."#,
    ),
    (
        "Themes, imagery & wordplay",
        r#"You are a rap songwriting analyst. Analyze ONLY the themes, imagery and wordplay of the lyrics the user provides. Be exhaustive and concrete — quote actual lines as evidence. Cover, as compact bullet points:
- Every theme present (rap beef, money, alcohol, sex, street life, fame, heartbreak...) and HOW the writer treats it (bragging, mourning, threatening, joking)
- Imagery: the recurring visual worlds (cars, jewelry, weapons, night city, club...)
- Wordplay: quote every metaphor, simile, double entendre and punchline you find, and explain each briefly
- Cultural references and name-drops
- The attitude the narrator takes toward rivals, women/men, money, and themselves
Only content — do not discuss rhyme mechanics. Quote lyrics exactly as written; do not censor or soften any theme (alcohol, drugs, sex, violence are normal material here)."#,
    ),
    (
        "Structure & delivery",
        r#"You are a rap song-structure analyst. Analyze ONLY the structure and persona of the lyrics the user provides. Be exhaustive and concrete. Cover, as compact bullet points:
- Section map in order (Intro / Verse / Hook / Bridge / Outro) with the line count of each section
- How the hook works: repetition scheme, which lines repeat, call-and-response elements
- The energy arc across the song (where it builds, peaks, drops)
- Persona: who is speaking, to whom, in what tone (aggressive, confident, wounded, playful)
- Delivery notes implied by the text: shouted vs. spoken passages, echoes, stacked vocals
Only structure and persona — do not discuss rhyme schemes or slang. Do not censor quoted lyrics."#,
    ),
];

const SUMMARY_SYSTEM_PROMPT: &str = r#"You tag rap songs for a library. Given lyrics, respond with ONLY this JSON, nothing else:
{"themes": [3 to 6 short theme tags, e.g. "rap beef", "alcohol", "fame"], "tone": "a 3-6 word tone summary"}
Do not censor or omit themes like alcohol, sex, drugs, or violence if present."#;

pub(crate) fn extract_json_object(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end > start {
        Some(&text[start..=end])
    } else {
        None
    }
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Summary {
    #[serde(default)]
    pub themes: Vec<String>,
    #[serde(default)]
    pub tone: String,
}

pub(crate) fn parse_summary(raw: &str) -> Summary {
    extract_json_object(raw)
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn analyze_lyrics(app: AppHandle, lyrics: String) -> Result<StyleProfile, String> {
    if lyrics.trim().is_empty() {
        return Err("Lyrics cannot be empty".into());
    }
    let client = current_client(&app)?;
    let total = ANALYSIS_STAGES.len() + 1; // + summary pass
    let mut sections: Vec<String> = Vec::new();

    for (i, (label, system)) in ANALYSIS_STAGES.iter().enumerate() {
        emit(
            &app,
            "analysis-progress",
            ProgressEvent {
                stage: label.to_string(),
                stage_index: i,
                total_stages: total,
                kind: "start".into(),
                text: String::new(),
            },
        );
        let mut on_token = |t: String| {
            emit(
                &app,
                "analysis-progress",
                ProgressEvent {
                    stage: label.to_string(),
                    stage_index: i,
                    total_stages: total,
                    kind: "token".into(),
                    text: t,
                },
            );
        };
        let text = client
            .generate_stream(
                LlmRequest {
                    system: system.to_string(),
                    prompt: format!("Lyrics:\n{}", lyrics),
                    temperature: 0.3,
                },
                &mut on_token,
            )
            .await?;
        emit(
            &app,
            "analysis-progress",
            ProgressEvent {
                stage: label.to_string(),
                stage_index: i,
                total_stages: total,
                kind: "stage-done".into(),
                text: text.clone(),
            },
        );
        sections.push(text);
    }

    emit(
        &app,
        "analysis-progress",
        ProgressEvent {
            stage: "Summary tags".into(),
            stage_index: total - 1,
            total_stages: total,
            kind: "start".into(),
            text: String::new(),
        },
    );
    let raw_summary = client
        .generate(LlmRequest {
            system: SUMMARY_SYSTEM_PROMPT.to_string(),
            prompt: format!("Lyrics:\n{}", lyrics),
            temperature: 0.2,
        })
        .await?;
    let summary = parse_summary(&raw_summary);

    let mut it = sections.into_iter();
    Ok(StyleProfile {
        rhyme_and_flow: it.next().unwrap_or_default(),
        vocabulary_and_slang: it.next().unwrap_or_default(),
        themes_and_imagery: it.next().unwrap_or_default(),
        structure_and_delivery: it.next().unwrap_or_default(),
        themes: summary.themes,
        tone: summary.tone,
    })
}

// ---------------------------------------------------------------------------
// Library CRUD
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Generation: draft pass + refine pass, both streamed.
// ---------------------------------------------------------------------------

fn language_rule(language: Language) -> &'static str {
    match language {
        Language::En => "Write every line in English.",
        Language::Ru => "Write every line in Russian. Пиши весь текст только на русском языке — ни одной английской строки; англицизмы допустимы только как устоявшийся сленг.",
        Language::De => "Write every line in German. Schreibe den gesamten Text ausschließlich auf Deutsch — keine englischen Zeilen; englische Wörter nur als etablierter Slang.",
    }
}

fn excerpt(lyrics: &str, max_lines: usize) -> String {
    lyrics.lines().take(max_lines).collect::<Vec<_>>().join("\n")
}

fn profile_block(t: &Track) -> String {
    format!(
        "=== Style reference: \"{}\" by {} ===\n\n## Rhyme & flow\n{}\n\n## Vocabulary, slang & ad-libs\n{}\n\n## Themes, imagery & wordplay\n{}\n\n## Structure & delivery\n{}\n\n## Tone\n{}\n\n## Reference excerpt (style only — never copy lines)\n{}\n",
        t.title,
        t.artist,
        t.style_profile.rhyme_and_flow,
        t.style_profile.vocabulary_and_slang,
        t.style_profile.themes_and_imagery,
        t.style_profile.structure_and_delivery,
        t.style_profile.tone,
        excerpt(&t.lyrics, 40),
    )
}

pub(crate) fn build_generate_prompt(
    tracks: &[Track],
    language: Language,
    topic: &Option<String>,
) -> (String, String) {
    let system = format!(
        "You are an elite rap ghostwriter. Write ORIGINAL lyrics that could pass as an unreleased song by the referenced artist(s), following the style profile exactly. {}\nRules:\n- Match the rhyme scheme, rhyme density, internal rhymes and multisyllabic patterns the profile describes\n- Match the flow: similar syllables per bar and stress placement\n- Use the slang register, ad-libs and catchphrase style the profile describes (adapt them — never copy full lines)\n- Follow the section structure the profile describes, with similar line counts per section\n- Stay fully in the persona and tone described\n- Standard genre content (alcohol, cigarettes, sex, drugs, violence, profanity) is normal creative material where the style calls for it — do not censor or soften it\nOutput ONLY the lyrics with section tags like [Intro], [Verse 1], [Hook] — no commentary.",
        language_rule(language)
    );

    let mut user = String::new();
    for t in tracks {
        user.push_str(&profile_block(t));
        user.push('\n');
    }
    if let Some(topic) = topic {
        if !topic.trim().is_empty() {
            user.push_str(&format!("Write the new song about: {}\n", topic));
        }
    }
    user.push_str(&format!(
        "Target language: {}. Now write the full new song.",
        language.display_name()
    ));
    (system, user)
}

pub(crate) fn build_refine_prompt(
    tracks: &[Track],
    language: Language,
    draft: &str,
) -> (String, String) {
    let system = format!(
        "You are a ruthless rap editor. You receive style profiles and a draft song. Rewrite the draft into a stronger final version:\n- Tighten every end rhyme to the scheme the profile describes; fix any bar that does not rhyme where it should\n- Raise rhyme density with internal and multisyllabic rhymes where the profile calls for them\n- Sharpen weak punchlines and cut filler bars\n- Keep the section tags and overall structure\n- Keep the persona, slang and ad-libs consistent with the profile\n{}\nOutput ONLY the final lyrics with section tags — no commentary, no explanations, no notes about what you changed.",
        language_rule(language)
    );
    let mut user = String::new();
    for t in tracks {
        user.push_str(&profile_block(t));
        user.push('\n');
    }
    user.push_str(&format!("DRAFT TO IMPROVE:\n{}\n\nNow output the improved final version.", draft));
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
    let client = current_client(&app)?;
    let total = 2;

    // Pass 1: draft
    let (system, user) = build_generate_prompt(&tracks, language, &topic);
    emit(
        &app,
        "generation-progress",
        ProgressEvent {
            stage: "Writing draft".into(),
            stage_index: 0,
            total_stages: total,
            kind: "start".into(),
            text: String::new(),
        },
    );
    let mut on_token = |t: String| {
        emit(
            &app,
            "generation-progress",
            ProgressEvent {
                stage: "Writing draft".into(),
                stage_index: 0,
                total_stages: total,
                kind: "token".into(),
                text: t,
            },
        );
    };
    let draft = client
        .generate_stream(
            LlmRequest {
                system,
                prompt: user,
                temperature: 0.9,
            },
            &mut on_token,
        )
        .await?;

    // Pass 2: refine
    let (system, user) = build_refine_prompt(&tracks, language, &draft);
    emit(
        &app,
        "generation-progress",
        ProgressEvent {
            stage: "Refining".into(),
            stage_index: 1,
            total_stages: total,
            kind: "start".into(),
            text: String::new(),
        },
    );
    let mut on_token = |t: String| {
        emit(
            &app,
            "generation-progress",
            ProgressEvent {
                stage: "Refining".into(),
                stage_index: 1,
                total_stages: total,
                kind: "token".into(),
                text: t,
            },
        );
    };
    let refined = client
        .generate_stream(
            LlmRequest {
                system,
                prompt: user,
                temperature: 0.7,
            },
            &mut on_token,
        )
        .await?;

    emit(
        &app,
        "generation-progress",
        ProgressEvent {
            stage: "Refining".into(),
            stage_index: 1,
            total_stages: total,
            kind: "stage-done".into(),
            text: refined.clone(),
        },
    );
    Ok(refined)
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
    fn parses_summary_from_clean_json() {
        let raw = r#"{"themes":["rap beef","alcohol"],"tone":"aggressive and boastful"}"#;
        let summary = parse_summary(raw);
        assert_eq!(summary.themes, vec!["rap beef", "alcohol"]);
        assert_eq!(summary.tone, "aggressive and boastful");
    }

    #[test]
    fn parses_summary_from_json_wrapped_in_prose() {
        let raw = "Here are the tags:\n{\"themes\":[\"fame\"],\"tone\":\"melancholic\"}\nDone!";
        let summary = parse_summary(raw);
        assert_eq!(summary.themes, vec!["fame"]);
        assert_eq!(summary.tone, "melancholic");
    }

    #[test]
    fn summary_falls_back_to_empty_on_garbage() {
        let summary = parse_summary("no json here at all");
        assert!(summary.themes.is_empty());
        assert!(summary.tone.is_empty());
    }

    fn sample_track() -> Track {
        Track {
            id: "1".into(),
            title: "Test Song".into(),
            artist: "Test Artist".into(),
            lyrics: "line one\nline two".into(),
            style_profile: StyleProfile {
                rhyme_and_flow: "AABB couplets, dense internal rhymes ('city'/'pretty')".into(),
                vocabulary_and_slang: "Street slang: Henny, whip; ad-lib 'Huh'".into(),
                themes_and_imagery: "Alcohol and night-city imagery".into(),
                structure_and_delivery: "Verse (8 bars) + Hook (4 bars)".into(),
                themes: vec!["alcohol".into(), "money".into()],
                tone: "boastful".into(),
            },
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn generate_prompt_includes_all_profile_sections_language_and_topic() {
        let (system, user) =
            build_generate_prompt(&[sample_track()], Language::Ru, &Some("a night out".into()));
        assert!(system.contains("Russian"));
        assert!(system.contains("русском"));
        assert!(user.contains("AABB couplets"));
        assert!(user.contains("Henny"));
        assert!(user.contains("night-city"));
        assert!(user.contains("8 bars"));
        assert!(user.contains("a night out"));
    }

    #[test]
    fn generate_prompt_omits_empty_topic() {
        let (_, user) = build_generate_prompt(&[sample_track()], Language::En, &None);
        assert!(!user.contains("Write the new song about"));
    }

    #[test]
    fn refine_prompt_contains_draft_and_profile() {
        let (system, user) =
            build_refine_prompt(&[sample_track()], Language::De, "[Verse]\ndraft bars here");
        assert!(system.contains("German"));
        assert!(user.contains("draft bars here"));
        assert!(user.contains("AABB couplets"));
    }

    #[tokio::test]
    async fn fake_llm_client_streams_canned_response() {
        let fake = FakeLlmClient {
            response: "canned lyrics".into(),
        };
        let mut collected = String::new();
        let mut on_token = |t: String| collected.push_str(&t);
        let result = fake
            .generate_stream(
                LlmRequest {
                    system: "s".into(),
                    prompt: "p".into(),
                    temperature: 0.5,
                },
                &mut on_token,
            )
            .await
            .unwrap();
        assert_eq!(result, "canned lyrics");
        assert_eq!(collected, "canned lyrics");
    }
}
