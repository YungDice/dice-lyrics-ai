use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{GenerationRecord, Settings, Track};

pub fn tracks_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("library").join("tracks")
}

pub fn generations_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("library").join("generations")
}

pub fn settings_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("settings.json")
}

fn ensure_dirs(app_data_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(tracks_dir(app_data_dir)).map_err(|e| e.to_string())?;
    fs::create_dir_all(generations_dir(app_data_dir)).map_err(|e| e.to_string())?;
    Ok(())
}

fn read_json_files<T: for<'de> serde::Deserialize<'de>>(dir: &Path) -> Result<Vec<T>, String> {
    let mut items = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(entry.path()).map_err(|e| e.to_string())?;
            items.push(serde_json::from_str(&content).map_err(|e| e.to_string())?);
        }
    }
    Ok(items)
}

pub fn save_track(app_data_dir: &Path, track: &Track) -> Result<(), String> {
    ensure_dirs(app_data_dir)?;
    let path = tracks_dir(app_data_dir).join(format!("{}.json", track.id));
    let json = serde_json::to_string_pretty(track).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn list_tracks(app_data_dir: &Path) -> Result<Vec<Track>, String> {
    ensure_dirs(app_data_dir)?;
    let mut tracks: Vec<Track> = read_json_files(&tracks_dir(app_data_dir))?;
    tracks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(tracks)
}

pub fn get_track(app_data_dir: &Path, id: &str) -> Result<Track, String> {
    let path = tracks_dir(app_data_dir).join(format!("{}.json", id));
    let content = fs::read_to_string(&path).map_err(|_| format!("Track {} not found", id))?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn delete_track(app_data_dir: &Path, id: &str) -> Result<(), String> {
    let path = tracks_dir(app_data_dir).join(format!("{}.json", id));
    fs::remove_file(path).map_err(|e| e.to_string())
}

pub fn save_generation(app_data_dir: &Path, generation: &GenerationRecord) -> Result<(), String> {
    ensure_dirs(app_data_dir)?;
    let path = generations_dir(app_data_dir).join(format!("{}.json", generation.id));
    let json = serde_json::to_string_pretty(generation).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn list_generations(app_data_dir: &Path) -> Result<Vec<GenerationRecord>, String> {
    ensure_dirs(app_data_dir)?;
    let mut generations: Vec<GenerationRecord> = read_json_files(&generations_dir(app_data_dir))?;
    generations.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(generations)
}

pub fn load_settings(app_data_dir: &Path) -> Settings {
    fs::read_to_string(settings_path(app_data_dir))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(app_data_dir: &Path, settings: &Settings) -> Result<(), String> {
    ensure_dirs(app_data_dir)?;
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(settings_path(app_data_dir), json).map_err(|e| e.to_string())
}
