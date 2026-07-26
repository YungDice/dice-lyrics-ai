import { invoke } from "@tauri-apps/api/core";

interface StyleProfile {
  rhymeScheme: string;
  cadence: string;
  themes: string[];
  vocabulary: string;
  structure: string;
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

type Language = "en" | "ru" | "de";

let tracks: Track[] = [];
let lastAnalyzedLyrics = "";
let lastGenerateArgs: { trackIds: string[]; language: Language; topic: string } | null = null;

// ---------- Navigation ----------

function showScreen(name: string) {
  document.querySelectorAll<HTMLElement>(".screen").forEach((el) => {
    el.classList.toggle("active", el.id === `screen-${name}`);
  });
  document.querySelectorAll<HTMLElement>(".navlink").forEach((el) => {
    el.classList.toggle("active", el.dataset.screen === name);
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
  const pill = document.querySelector<HTMLElement>("#ollama-status")!;
  const text = document.querySelector<HTMLElement>("#ollama-status-text")!;
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
  const el = document.querySelector<HTMLElement>("#analyze-error")!;
  if (message) {
    el.textContent = message;
    el.hidden = false;
  } else {
    el.hidden = true;
  }
}

function fillStyleProfileForm(profile: StyleProfile) {
  (document.querySelector("#sp-rhyme") as HTMLTextAreaElement).value = profile.rhymeScheme;
  (document.querySelector("#sp-cadence") as HTMLTextAreaElement).value = profile.cadence;
  (document.querySelector("#sp-themes") as HTMLInputElement).value = profile.themes.join(", ");
  (document.querySelector("#sp-tone") as HTMLInputElement).value = profile.tone;
  (document.querySelector("#sp-vocabulary") as HTMLTextAreaElement).value = profile.vocabulary;
  (document.querySelector("#sp-structure") as HTMLTextAreaElement).value = profile.structure;
  document.querySelector<HTMLElement>("#analyze-result")!.hidden = false;
}

function readStyleProfileForm(): StyleProfile {
  return {
    rhymeScheme: (document.querySelector("#sp-rhyme") as HTMLTextAreaElement).value,
    cadence: (document.querySelector("#sp-cadence") as HTMLTextAreaElement).value,
    themes: (document.querySelector("#sp-themes") as HTMLInputElement).value
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean),
    vocabulary: (document.querySelector("#sp-vocabulary") as HTMLTextAreaElement).value,
    structure: (document.querySelector("#sp-structure") as HTMLTextAreaElement).value,
    tone: (document.querySelector("#sp-tone") as HTMLInputElement).value,
  };
}

async function handleAnalyze() {
  const lyrics = (document.querySelector("#analyze-lyrics") as HTMLTextAreaElement).value;
  showAnalyzeError(null);
  if (!lyrics.trim()) {
    showAnalyzeError("Paste some lyrics first.");
    return;
  }
  const btn = document.querySelector<HTMLButtonElement>("#analyze-btn")!;
  const spinner = document.querySelector<HTMLElement>("#analyze-spinner")!;
  btn.disabled = true;
  spinner.hidden = false;
  document.querySelector<HTMLElement>("#analyze-result")!.hidden = true;
  try {
    const profile = await invoke<StyleProfile>("analyze_lyrics", { lyrics });
    lastAnalyzedLyrics = lyrics;
    fillStyleProfileForm(profile);
  } catch (err) {
    showAnalyzeError(String(err));
  } finally {
    btn.disabled = false;
    spinner.hidden = true;
  }
}

async function handleSaveTrack() {
  const title = (document.querySelector("#analyze-title") as HTMLInputElement).value || "Untitled";
  const artist = (document.querySelector("#analyze-artist") as HTMLInputElement).value || "Unknown";
  const styleProfile = readStyleProfileForm();
  try {
    await invoke<Track>("save_track", {
      title,
      artist,
      lyrics: lastAnalyzedLyrics,
      styleProfile,
    });
    showAnalyzeError(null);
    (document.querySelector("#analyze-title") as HTMLInputElement).value = "";
    (document.querySelector("#analyze-artist") as HTMLInputElement).value = "";
    (document.querySelector("#analyze-lyrics") as HTMLTextAreaElement).value = "";
    document.querySelector<HTMLElement>("#analyze-result")!.hidden = true;
    showScreen("library");
  } catch (err) {
    showAnalyzeError(String(err));
  }
}

// ---------- Library screen ----------

async function refreshLibrary() {
  const container = document.querySelector<HTMLElement>("#library-list")!;
  try {
    tracks = await invoke<Track[]>("list_tracks");
    renderLibrary();
  } catch (err) {
    container.innerHTML = `<div class="empty-state">Could not load your library: ${escapeHtml(String(err))}</div>`;
  }
}

function renderLibrary() {
  const container = document.querySelector<HTMLElement>("#library-list")!;
  const query = (document.querySelector("#library-search") as HTMLInputElement).value.toLowerCase();
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
  const container = document.querySelector<HTMLElement>("#generate-ref-list")!;
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
  const el = document.querySelector<HTMLElement>("#generate-error")!;
  if (message) {
    el.textContent = message;
    el.hidden = false;
  } else {
    el.hidden = true;
  }
}

async function runGenerate(trackIds: string[], language: Language, topic: string) {
  const btn = document.querySelector<HTMLButtonElement>("#generate-btn")!;
  const spinner = document.querySelector<HTMLElement>("#generate-spinner")!;
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
    const outputWrap = document.querySelector<HTMLElement>("#generate-output-wrap")!;
    document.querySelector<HTMLElement>("#generate-output")!.textContent = lyrics;
    outputWrap.hidden = false;
  } catch (err) {
    showGenerateError(String(err));
  } finally {
    btn.disabled = false;
    spinner.hidden = true;
  }
}

async function handleGenerate() {
  const trackIds = Array.from(
    document.querySelectorAll<HTMLInputElement>("#generate-ref-list input:checked")
  ).map((el) => el.value);
  if (trackIds.length === 0) {
    showGenerateError("Select at least one reference track.");
    return;
  }
  const language = (document.querySelector("input[name=language]:checked") as HTMLInputElement)
    .value as Language;
  const topic = (document.querySelector("#generate-topic") as HTMLInputElement).value;
  await runGenerate(trackIds, language, topic);
}

async function handleRegenerate() {
  if (!lastGenerateArgs) return;
  await runGenerate(lastGenerateArgs.trackIds, lastGenerateArgs.language, lastGenerateArgs.topic);
}

async function handleCopy() {
  const text = document.querySelector<HTMLElement>("#generate-output")!.textContent ?? "";
  await navigator.clipboard.writeText(text);
}

async function handleSaveGeneration() {
  if (!lastGenerateArgs) return;
  const lyrics = document.querySelector<HTMLElement>("#generate-output")!.textContent ?? "";
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

  document.querySelector("#analyze-btn")!.addEventListener("click", () => void handleAnalyze());
  document.querySelector("#save-track-btn")!.addEventListener("click", () => void handleSaveTrack());
  document.querySelector("#library-search")!.addEventListener("input", () => renderLibrary());
  document.querySelector("#generate-btn")!.addEventListener("click", () => void handleGenerate());
  document.querySelector("#regenerate-btn")!.addEventListener("click", () => void handleRegenerate());
  document.querySelector("#copy-btn")!.addEventListener("click", () => void handleCopy());
  document
    .querySelector("#save-generation-btn")!
    .addEventListener("click", () => void handleSaveGeneration());
});
