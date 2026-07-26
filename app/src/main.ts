import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface StyleProfile {
  rhymeAndFlow: string;
  vocabularyAndSlang: string;
  themesAndImagery: string;
  structureAndDelivery: string;
  themes: string[];
  tone: string;
}

interface Track {
  id: string;
  title: string;
  artist: string;
  lyrics: string;
  styleProfile: StyleProfile;
  createdAt: string;
}

interface ProgressEvent {
  stage: string;
  stageIndex: number;
  totalStages: number;
  kind: "start" | "token" | "stage-done";
  text: string;
}

type Language = "en" | "ru" | "de";

let tracks: Track[] = [];
let lastAnalyzedLyrics = "";
let lastGenerateArgs: { trackIds: string[]; language: Language; topic: string } | null = null;

// Analysis stages 0-3 stream into these textareas, in backend stage order.
const ANALYSIS_STAGE_FIELDS = ["sp-rhyme-flow", "sp-vocab", "sp-themes-imagery", "sp-structure"];

function el<T extends HTMLElement>(selector: string): T {
  return document.querySelector(selector) as T;
}

// ---------- Navigation ----------

function showScreen(name: string) {
  document.querySelectorAll<HTMLElement>(".screen").forEach((e) => {
    e.classList.toggle("active", e.id === `screen-${name}`);
  });
  document.querySelectorAll<HTMLElement>(".navlink").forEach((e) => {
    e.classList.toggle("active", e.dataset.screen === name);
  });
  if (name === "library") {
    void refreshLibrary();
  }
  if (name === "generate") {
    void refreshReferenceList();
  }
}

function setupNav() {
  document.querySelectorAll<HTMLButtonElement>(".navlink").forEach((btn) => {
    btn.addEventListener("click", () => showScreen(btn.dataset.screen!));
  });
}

// ---------- Ollama status ----------

async function checkOllamaStatus() {
  const pill = el("#ollama-status");
  const text = el("#ollama-status-text");
  try {
    const ok = await invoke<boolean>("check_ollama_status");
    pill.classList.toggle("ok", ok);
    pill.classList.toggle("down", !ok);
    text.textContent = ok ? "Ollama running" : "Ollama not running";
  } catch {
    pill.classList.add("down");
    text.textContent = "Ollama not running";
  }
}

// ---------- Analyze screen ----------

function showAnalyzeError(message: string | null) {
  const e = el("#analyze-error");
  if (message) {
    e.textContent = message;
    e.hidden = false;
  } else {
    e.hidden = true;
  }
}

function onAnalysisProgress(ev: ProgressEvent) {
  const stageLabel = el("#analyze-stage");
  stageLabel.hidden = false;
  stageLabel.textContent = `Analyzing (${ev.stageIndex + 1}/${ev.totalStages}): ${ev.stage}…`;

  const fieldId = ANALYSIS_STAGE_FIELDS[ev.stageIndex];
  if (!fieldId) return; // summary stage has no streaming field
  const area = el<HTMLTextAreaElement>(`#${fieldId}`);
  if (ev.kind === "start") {
    el("#analyze-result").hidden = false;
    area.value = "";
  } else if (ev.kind === "token") {
    area.value += ev.text;
    area.scrollTop = area.scrollHeight;
  } else if (ev.kind === "stage-done") {
    area.value = ev.text;
  }
}

function fillStyleProfileForm(profile: StyleProfile) {
  el<HTMLTextAreaElement>("#sp-rhyme-flow").value = profile.rhymeAndFlow;
  el<HTMLTextAreaElement>("#sp-vocab").value = profile.vocabularyAndSlang;
  el<HTMLTextAreaElement>("#sp-themes-imagery").value = profile.themesAndImagery;
  el<HTMLTextAreaElement>("#sp-structure").value = profile.structureAndDelivery;
  el<HTMLInputElement>("#sp-themes").value = profile.themes.join(", ");
  el<HTMLInputElement>("#sp-tone").value = profile.tone;
  el("#analyze-result").hidden = false;
}

function readStyleProfileForm(): StyleProfile {
  return {
    rhymeAndFlow: el<HTMLTextAreaElement>("#sp-rhyme-flow").value,
    vocabularyAndSlang: el<HTMLTextAreaElement>("#sp-vocab").value,
    themesAndImagery: el<HTMLTextAreaElement>("#sp-themes-imagery").value,
    structureAndDelivery: el<HTMLTextAreaElement>("#sp-structure").value,
    themes: el<HTMLInputElement>("#sp-themes").value
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean),
    tone: el<HTMLInputElement>("#sp-tone").value,
  };
}

async function handleAnalyze() {
  const lyrics = el<HTMLTextAreaElement>("#analyze-lyrics").value;
  showAnalyzeError(null);
  if (!lyrics.trim()) {
    showAnalyzeError("Paste some lyrics first.");
    return;
  }
  const btn = el<HTMLButtonElement>("#analyze-btn");
  const spinner = el("#analyze-spinner");
  btn.disabled = true;
  spinner.hidden = false;
  for (const id of ANALYSIS_STAGE_FIELDS) {
    el<HTMLTextAreaElement>(`#${id}`).value = "";
  }
  el<HTMLInputElement>("#sp-themes").value = "";
  el<HTMLInputElement>("#sp-tone").value = "";
  try {
    const profile = await invoke<StyleProfile>("analyze_lyrics", { lyrics });
    lastAnalyzedLyrics = lyrics;
    fillStyleProfileForm(profile);
    el("#analyze-stage").textContent = "Analysis complete — review and edit below, then save.";
  } catch (err) {
    showAnalyzeError(String(err));
    el("#analyze-stage").hidden = true;
  } finally {
    btn.disabled = false;
    spinner.hidden = true;
  }
}

async function handleSaveTrack() {
  const title = el<HTMLInputElement>("#analyze-title").value || "Untitled";
  const artist = el<HTMLInputElement>("#analyze-artist").value || "Unknown";
  const styleProfile = readStyleProfileForm();
  try {
    await invoke<Track>("save_track", {
      title,
      artist,
      lyrics: lastAnalyzedLyrics,
      styleProfile,
    });
    showAnalyzeError(null);
    el<HTMLInputElement>("#analyze-title").value = "";
    el<HTMLInputElement>("#analyze-artist").value = "";
    el<HTMLTextAreaElement>("#analyze-lyrics").value = "";
    el("#analyze-result").hidden = true;
    el("#analyze-stage").hidden = true;
    showScreen("library");
  } catch (err) {
    showAnalyzeError(String(err));
  }
}

// ---------- Library screen ----------

async function refreshLibrary() {
  const container = el("#library-list");
  try {
    tracks = await invoke<Track[]>("list_tracks");
    renderLibrary();
  } catch (err) {
    container.innerHTML = `<div class="empty-state">Could not load your library: ${escapeHtml(String(err))}</div>`;
  }
}

function renderLibrary() {
  const container = el("#library-list");
  const query = el<HTMLInputElement>("#library-search").value.toLowerCase();
  const filtered = tracks.filter(
    (t) => t.title.toLowerCase().includes(query) || t.artist.toLowerCase().includes(query)
  );

  if (filtered.length === 0) {
    container.innerHTML = `<div class="empty-state">No analyzed songs yet. Head to Analyze to add one.</div>`;
    return;
  }

  container.innerHTML = "";
  for (const track of filtered) {
    const row = document.createElement("div");
    row.className = "track-row";
    row.innerHTML = `
      <div class="track-row__main">
        <div>
          <div class="track-row__title">${escapeHtml(track.title)}</div>
          <div class="track-row__artist">${escapeHtml(track.artist)}</div>
        </div>
        <div class="track-row__themes">${escapeHtml(track.styleProfile.themes.join(", "))}</div>
      </div>
      <button class="pill small" data-delete="${track.id}">Delete</button>
    `;
    container.appendChild(row);
  }

  container.querySelectorAll<HTMLButtonElement>("[data-delete]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      await invoke("delete_track", { id: btn.dataset.delete });
      await refreshLibrary();
    });
  });
}

function escapeHtml(s: string): string {
  const div = document.createElement("div");
  div.textContent = s;
  return div.innerHTML;
}

// ---------- Generate screen ----------

async function refreshReferenceList() {
  const container = el("#generate-ref-list");
  try {
    tracks = await invoke<Track[]>("list_tracks");
  } catch (err) {
    container.innerHTML = `<div class="empty-state">Could not load your library: ${escapeHtml(String(err))}</div>`;
    return;
  }
  if (tracks.length === 0) {
    container.innerHTML = `<div class="empty-state">No analyzed songs yet. Analyze one first.</div>`;
    return;
  }
  container.innerHTML = tracks
    .map(
      (t) => `
      <label class="ref-item">
        <input type="checkbox" value="${t.id}" />
        <span>${escapeHtml(t.title)} — ${escapeHtml(t.artist)}</span>
      </label>
    `
    )
    .join("");
}

function showGenerateError(message: string | null) {
  const e = el("#generate-error");
  if (message) {
    e.textContent = message;
    e.hidden = false;
  } else {
    e.hidden = true;
  }
}

function onGenerationProgress(ev: ProgressEvent) {
  const stageLabel = el("#generate-stage");
  stageLabel.hidden = false;
  stageLabel.textContent = `${ev.stage} (${ev.stageIndex + 1}/${ev.totalStages})…`;

  const output = el("#generate-output");
  if (ev.kind === "start") {
    el("#generate-output-wrap").hidden = false;
    output.textContent = "";
  } else if (ev.kind === "token") {
    output.textContent += ev.text;
  } else if (ev.kind === "stage-done") {
    output.textContent = ev.text;
  }
}

async function runGenerate(trackIds: string[], language: Language, topic: string) {
  const btn = el<HTMLButtonElement>("#generate-btn");
  const spinner = el("#generate-spinner");
  btn.disabled = true;
  spinner.hidden = false;
  showGenerateError(null);
  try {
    const lyrics = await invoke<string>("generate_lyrics", {
      trackIds,
      language,
      topic: topic || null,
    });
    lastGenerateArgs = { trackIds, language, topic };
    el("#generate-output").textContent = lyrics;
    el("#generate-output-wrap").hidden = false;
    el("#generate-stage").textContent = "Done.";
  } catch (err) {
    showGenerateError(String(err));
    el("#generate-stage").hidden = true;
  } finally {
    btn.disabled = false;
    spinner.hidden = true;
  }
}

async function handleGenerate() {
  const trackIds = Array.from(
    document.querySelectorAll<HTMLInputElement>("#generate-ref-list input:checked")
  ).map((e) => e.value);
  if (trackIds.length === 0) {
    showGenerateError("Select at least one reference track.");
    return;
  }
  const language = (document.querySelector("input[name=language]:checked") as HTMLInputElement)
    .value as Language;
  const topic = el<HTMLInputElement>("#generate-topic").value;
  await runGenerate(trackIds, language, topic);
}

async function handleRegenerate() {
  if (!lastGenerateArgs) return;
  await runGenerate(lastGenerateArgs.trackIds, lastGenerateArgs.language, lastGenerateArgs.topic);
}

async function handleCopy() {
  const text = el("#generate-output").textContent ?? "";
  await navigator.clipboard.writeText(text);
}

async function handleSaveGeneration() {
  if (!lastGenerateArgs) return;
  const lyrics = el("#generate-output").textContent ?? "";
  await invoke("save_generation", {
    trackIds: lastGenerateArgs.trackIds,
    language: lastGenerateArgs.language,
    topic: lastGenerateArgs.topic || null,
    lyrics,
  });
}

// ---------- Wiring ----------

window.addEventListener("DOMContentLoaded", () => {
  setupNav();
  void checkOllamaStatus();
  setInterval(checkOllamaStatus, 15000);

  void listen<ProgressEvent>("analysis-progress", (e) => onAnalysisProgress(e.payload));
  void listen<ProgressEvent>("generation-progress", (e) => onGenerationProgress(e.payload));

  el("#analyze-btn").addEventListener("click", () => void handleAnalyze());
  el("#save-track-btn").addEventListener("click", () => void handleSaveTrack());
  el("#library-search").addEventListener("input", () => renderLibrary());
  el("#generate-btn").addEventListener("click", () => void handleGenerate());
  el("#regenerate-btn").addEventListener("click", () => void handleRegenerate());
  el("#copy-btn").addEventListener("click", () => void handleCopy());
  el("#save-generation-btn").addEventListener("click", () => void handleSaveGeneration());
});
