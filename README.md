# Dice Lyrics AI

A Windows desktop app that analyzes rap lyrics into a style profile, then
generates new original lyrics in that style — in English, Russian, or German —
using a local, uncensored LLM (no cloud calls, no content filtering on
genre-standard themes).

See [docs/research/uncensored-local-llm-for-lyrics.md](docs/research/uncensored-local-llm-for-lyrics.md)
for why `dolphin-mistral:7b` was chosen, and [DESIGN.md](DESIGN.md) for the
visual design system.

## Prerequisites

- [Ollama](https://ollama.com) installed and running
- The model pulled: `ollama pull dolphin-mistral:7b`
- Node.js and the Rust toolchain (only needed to build from source)

## Running from source

```bash
cd app
npm install
npm run tauri dev
```

## Building a release installer

```bash
cd app
npm run tauri build
```

The installer and standalone `.exe` are written to
`app/src-tauri/target/release/bundle/`.

## How to use it

1. **Analyze** — paste a song's lyrics, optionally add a title/artist, and
   click Analyze. Review/edit the extracted style profile, then save it to
   your Library.
2. **Library** — browse, search, and delete the songs you've analyzed.
3. **Generate** — pick one or more analyzed songs as a style reference,
   choose a language, optionally give a topic, and generate new lyrics.
   Copy, save, or regenerate from the same settings.

Settings (Ollama URL and model) are stored in `settings.json` under the app's
data directory; analyzed tracks and generations are stored as individual JSON
files under `library/tracks/` and `library/generations/` in the same
directory.
