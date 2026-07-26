# Research: Uncensored Local LLM for Rap Lyrics Analysis & Generation

**Scope:** Identify a locally-run, uncensored LLM (via Ollama, run on a Windows desktop) suitable for (1) extracting a style profile from pasted rap lyrics and (2) generating new lyrics in that style, in English, Russian, or German. All claims below are sourced from primary sources (Ollama's own library pages, Hugging Face model cards, official Ollama docs, Meta's/Mistral's/Google's own license text) fetched directly — not secondary blog summaries. Each finding links to the exact page it came from.

---

## 1. Ollama model library: what "uncensored" models actually exist today

Searching [ollama.com/search?q=uncensored](https://ollama.com/search?q=uncensored) and [ollama.com/search?q=abliterated](https://ollama.com/search?q=abliterated) directly (fetched live) surfaces two distinct families:

### A. The "Dolphin" family (Eric Hartford / Cognitive Computations) — official Ollama library models

| Model tag | Sizes | Base model | Context | "Uncensored" per its own card |
|---|---|---|---|---|
| [dolphin-mixtral](https://ollama.com/library/dolphin-mixtral) | 8x7b, 8x22b | Mixtral (MoE) | 32K (8x7b) / 64K (8x22b) | "Uncensored... I have filtered the dataset to remove alignment and bias. This makes the model more compliant." Explicit warning: "It will be highly compliant to any requests, even unethical ones," recommends the *deployer* add their own alignment layer.[^dm][^dm-hf] |
| [dolphin-mistral](https://ollama.com/library/dolphin-mistral) | 7b | Mistral 0.2 | 32K | Same Dolphin methodology; page states model is "available for both commercial and non-commercial use."[^dmi] |
| [dolphin-llama3](https://ollama.com/library/dolphin-llama3) | 8b, 70b | Llama 3 | 8K (256K variant needs 64GB+ RAM) | Same methodology: "the dataset has been filtered to remove alignment and bias."[^dl3][^dl3-hf] |
| [llama2-uncensored](https://ollama.com/library/llama2-uncensored) | 7b, 70b | Llama 2 | 2K | Built "using the process defined by Eric Hartford in his blog post" on uncensored models.[^l2u] |
| [wizardlm-uncensored](https://ollama.com/library/wizardlm-uncensored) | 13b | Llama 2 | 4K | Trained on a dataset where "responses that contained alignment/moralizing were removed."[^wlu] |
| [wizard-vicuna-uncensored](https://ollama.com/search?q=uncensored) | 7b, 13b, 30b | Llama 2 | — | Same Hartford-style filtering; listed in Ollama search results.[^search] |
| [everythinglm](https://ollama.com/search?q=uncensored) | 13b | Llama 2 | 16K | "Uncensored Llama2 based model with support for a 16K context window."[^search] |

All Dolphin-family cards use nearly identical language: this is **dataset-level decensoring** — the fine-tuning corpus itself was filtered to strip refusals/moralizing before training, as opposed to a post-hoc weight edit. Hartford's own cards are explicit that this makes the model *fully* compliant, not "reduced guardrails" — they warn operators to add their own safety layer if needed, implying none is baked in.

### B. "Abliterated" models — community-namespace uploads, dominated by `huihui_ai`

Abliteration is a different, newer technique: a post-hoc activation-directional weight edit that suppresses the model's internal "refusal direction" without retraining on filtered data (originating from the `remove-refusals-with-transformers` technique, referenced directly in the huihui_ai Qwen model card).[^qwen-abl] Ollama's abliterated search results are dominated by one publisher, `huihui_ai` (also on Hugging Face as `huihui-ai`, a PRO account, 235+ published models, self-described focus on "model ablations"):[^huihui]

| Model | Sizes | Base | Pulls | Notes |
|---|---|---|---|---|
| [huihui_ai/qwen3.5-abliterated](https://ollama.com/huihui_ai/qwen3.5-abliterated) | 0.8B–122B | Qwen 3.5 | 449.5K | Ollama's own card carries an explicit caution: **"risk of sensitive or controversial outputs," "not suitable for all audiences,"** recommends restricting to "research and controlled environments rather than production deployment."[^qwen-abl] |
| [huihui_ai/gemma-4-abliterated](https://ollama.com/huihui_ai/gemma-4-abliterated) | e2b/e4b/12b/26b/31b/48b | Gemma 4 | 743.6K | Most-pulled abliterated model on Ollama currently. |
| huihui_ai/Qwen3.6-abliterated | 27b, 35b | Qwen 3.6 | 261.8K | Newer Qwen generation. |

Other abliterated entries in the search results (e.g., `feadxus/*`, `aratan/*`, `lileilei999/*`) are small-follower individual re-uploads with negligible pull counts (single/double digits to low thousands) — not vetted or maintained enough to recommend for a shipping product.

**Key distinction for this project:** Dolphin models are trained by their author to be uncensored and Ollama's/HF's own text for them contains no production-use disclaimer. The `huihui_ai` abliterated Qwen card, by contrast, contains Ollama-hosted language explicitly discouraging production deployment — a meaningful primary-source signal against building a shipping app on it without further scrutiny.

---

## 2. Licensing

Checked directly against Ollama pages and the underlying Hugging Face model cards (Ollama pages frequently omit license text; HF cards are authoritative):

| Model family | Base license | Redistribution terms relevant to a packaged Windows app |
|---|---|---|
| **dolphin-mixtral** (Mixtral base) | **Apache 2.0** — confirmed on [mistralai/Mixtral-8x7B-Instruct-v0.1](https://huggingface.co/mistralai/Mixtral-8x7B-Instruct-v0.1) ("License: apache-2.0")[^mixtral-hf] | Fully permissive: commercial and personal redistribution, no attribution notice requirement, no user-cap clause. |
| **dolphin-mistral** (Mistral 0.2 base) | Apache 2.0 (Mistral's standard license family; Ollama's own page states the model is "available for both commercial and non-commercial use")[^dmi] | Same as above. |
| **dolphin-llama3** (Llama 3 base) | **Meta Llama 3 Community License** — confirmed on [meta-llama/Meta-Llama-3-8B-Instruct](https://huggingface.co/meta-llama/Meta-Llama-3-8B-Instruct)[^l3-hf] | Not simply permissive: requires displaying **"Built with Meta Llama 3"**, requires including a copy of the license with the app, prohibits using outputs "to improve any other large language model," and requires a separate license from Meta if the app ever exceeds **700M monthly active users**. Workable for an indie Windows app but adds real redistribution obligations Apache-licensed models don't have. |
| **llama2-uncensored / wizardlm-uncensored / wizard-vicuna-uncensored** (Llama 2 base) | Llama 2 Community License (same family of restrictions as Llama 3: attribution, MAU clause, acceptable-use policy flow-down) | Same category of obligations as dolphin-llama3, on an older/weaker base model. |
| **huihui_ai/qwen3.5-abliterated** (Qwen 3.5 base) | Qwen 3.5 base itself is **Apache 2.0** — confirmed on [Qwen/Qwen3.5-35B-A3B](https://huggingface.co/Qwen/Qwen3.5-35B-A3B) ("License: apache-2.0")[^qwen-license] | Abliteration is a weight edit, not a relicensing event, so in principle the Apache 2.0 terms carry forward — but the specific `huihui-ai` HF repo for the exact abliterated checkpoint returned **HTTP 401 (gated/auth-required)**, so its own license field could not be directly verified as a primary source. Note this gap rather than assume. |
| **huihui_ai/gemma-4-abliterated** (Gemma base) | **Custom Gemma Terms of Use**, confirmed on [ai.google.dev/gemma/terms](https://ai.google.dev/gemma/terms)[^gemma-terms] | Meaningfully more restrictive than Apache: redistribution requires you to (a) pass the full Gemma Terms of Use to your app's end users, (b) include a notice ("Gemma is provided under and subject to the Gemma Terms of Use..."), (c) mark any modifications, and (d) bind your own users to the Gemma Prohibited Use Policy. This flow-down obligation is a real integration burden for a consumer app installer. |

**Bottom line:** dolphin-mixtral and dolphin-mistral are the only candidates researched here with a clean, fully permissive Apache 2.0 license and no attribution/notice/MAU obligations. Everything Llama- or Gemma-based carries redistribution paperwork.

---

## 3. Hardware / VRAM requirements

Ollama's own [GPU documentation](https://github.com/ollama/ollama/blob/main/docs/gpu.mdx) explicitly states it does **not** publish fixed VRAM sizing tables — it describes the scheduler's behavior ("leverages available VRAM data reported by the GPU libraries to make optimal scheduling decisions," falling back to "approximate sizes of the models" when it can't query hardware directly) but gives no capacity-planning numbers.[^gpu-doc] The most reliable primary-source proxy is each model's **on-disk quantized file size** from Ollama's own tags pages, since Ollama needs roughly that much RAM+VRAM combined to hold the weights, plus overhead for context/KV-cache:

| Model : tag | Q4_K_M size (Ollama tags page) | Practical implication on a consumer Windows PC |
|---|---|---|
| dolphin-mistral:7b | 4.1 GB (default quant)[^dmi] | Runs comfortably on an 8 GB GPU, or CPU-only with 16 GB system RAM. |
| dolphin-llama3:8b | 4.7 GB[^dl3] | Similar to above — one of the most accessible options. |
| llama2-uncensored:7b | 3.8 GB[^l2u] | Very light; but see context-window limitation below. |
| wizardlm-uncensored:13b | 7.4 GB (default 4-bit); Ollama states **minimum 16 GB RAM**[^wlu] | Mid-range consumer GPU (12–16 GB VRAM) or CPU+32GB RAM. |
| dolphin-mixtral:8x7b | q4_K_M = 28 GB[^dm-tags] | Needs a 24 GB+ VRAM GPU (e.g. RTX 3090/4090) for full GPU offload, or 32 GB+ system RAM for CPU/partial-offload (slower). Not "typical consumer PC" territory without a high-end GPU. |
| dolphin-mixtral:8x22b | q4_K_M = 86 GB[^dm-tags] | Workstation/server-class hardware only — out of scope for a consumer Windows app. |
| huihui_ai/qwen3.5-abliterated | 0.8B (1.0GB) up to 122B (81GB); mid sizes: 4B=3.3GB, 9B=6.6GB, 27B=17GB[^qwen-abl] | The 4B–9B tiers are consumer-friendly; 27B+ needs a high-VRAM GPU. |
| llama2-uncensored:70b / dolphin-llama3:70b | ~39–40 GB[^l2u][^dl3] | Ollama states **minimum 64 GB RAM** for the 70b tier — enthusiast/workstation hardware. |

For a "typical consumer Windows PC" target, the 7B–8B tier (dolphin-mistral, dolphin-llama3, or Qwen3.5-abliterated at 4B/9B) is the realistic default; the 8x7B Mixtral tier is a "if the user has a 24GB+ GPU" upgrade path, not a baseline requirement.

---

## 4. Multilingual quality (Russian / German specifically)

This is the area primary sources are **weakest** on — most cards are silent on anything beyond English, and I'm flagging that silence rather than inferring quality:

- **Dolphin family:** None of the Ollama pages or the underlying Cognitive Computations Hugging Face cards ([dolphin-2.9-llama3-8b](https://huggingface.co/cognitivecomputations/dolphin-2.9-llama3-8b), [dolphin-2.7-mixtral-8x7b](https://huggingface.co/cognitivecomputations/dolphin-2.7-mixtral-8x7b)) mention Russian or German. The Mixtral card explicitly lists supported languages as a number ("5 languages") without naming them.[^mixtral-hf] The Dolphin-2.7-Mixtral card explicitly names **only English** as a supported/tested language.[^dm-hf]
- **Llama 3 base (used by dolphin-llama3): important negative finding.** Meta's own [Meta-Llama-3-8B-Instruct](https://huggingface.co/meta-llama/Meta-Llama-3-8B-Instruct) card states the model is **"intended for commercial and research use in English"** and explicitly lists **"Use in languages other than English"** under **Out-of-Scope Uses**.[^l3-hf] This is a direct primary-source statement that dolphin-llama3's base model is not designed for Russian/German at all — a meaningful strike against it for this app's multilingual requirement.
- **Qwen 3.5 (base for huihui_ai's abliterated variant):** The official [Qwen/Qwen3.5-35B-A3B](https://huggingface.co/Qwen/Qwen3.5-35B-A3B) card states **"Expanded support to 201 languages and dialects, enabling inclusive, worldwide deployment."**[^qwen-license] This is a broad, generic claim — the card does **not** break out Russian or German by name, so I cannot confirm specific fluency, only that the vendor claims broad multilingual coverage as a design goal (Qwen models have historically been trained on large multilingual corpora, consistent with this generic claim, but the card itself gives no per-language benchmark).
- **Mistral (base for dolphin-mistral):** Neither Ollama's page nor the Mistral license/card material fetched mentions specific language support; Mistral models are widely known to have reasonable European-language competence from pretraining data composition, but no primary source checked here makes an explicit Russian/German claim.

**Conclusion for this section:** if Russian and German output quality matters as much as English, **Qwen3.5-based models (huihui_ai's abliterated builds) are the only candidate family whose own vendor makes an explicit, broad multilingual claim** — though not a granular one. Llama-3-based Dolphin models are actively contraindicated by Meta's own out-of-scope language statement. Mixtral/Mistral-based Dolphin models are simply undocumented on this axis, not disqualified.

---

## 5. Architecture for style-conditioned generation (Ollama does not fine-tune)

Ollama's own docs confirm it is not a training/fine-tuning platform in the traditional sense:

- **[Modelfile reference](https://github.com/ollama/ollama/blob/main/docs/modelfile.mdx):** supports `SYSTEM` (sets the persistent system message), `TEMPLATE` (Go-template prompt structure with `{{ .System }}`, `{{ .Prompt }}`, `{{ .Response }}` variables), `PARAMETER num_ctx` (context window size, default **2048 tokens** unless overridden), and `MESSAGE` (lets you bake in example system/user/assistant turns — i.e., **few-shot examples directly into a custom model definition**).[^modelfile] The only training-adjacent instruction is `ADAPTER`, which **applies** a pre-trained LoRA adapter (Safetensors or GGUF) to a base model — Ollama does not create or train adapters itself; that has to happen upstream (e.g., via a separate fine-tuning toolchain) before the adapter is imported.[^modelfile]
- **[Context length docs](https://github.com/ollama/ollama/blob/main/docs/context-length.mdx):** Ollama auto-sizes context based on detected VRAM (under 24GB VRAM → 4,000 tokens; 24–48GB → 32,000; 48GB+ → 256,000), overridable via the `OLLAMA_CONTEXT_LENGTH` environment variable or an app settings slider, with an explicit warning that "setting a larger context length will increase the amount of memory required."[^ctxlen]

Given this, the realistic architecture for this app is **(a) prompt engineering / few-shot with the analyzed style profile(s) injected as context**, not (c) fine-tuning:

- Store each analyzed song's extracted style profile (rhyme scheme, cadence notes, themes, slang register, structure) as structured text/JSON.
- At generation time, build a prompt via a `SYSTEM` instruction (persistent style/role instructions) plus `MESSAGE`-style or inline few-shot excerpts from the selected reference song(s)/artist(s), within the model's context window.
- (b) RAG-style retrieval only becomes necessary if the user's analyzed-song library grows large enough that not all relevant profiles fit in context at once (e.g., selecting "artist X" pulls in 40 analyzed songs) — at that point, retrieve the top-N most relevant stored analyses (by embedding similarity or simple metadata filtering) and inject only those into the prompt, rather than fine-tuning per-artist. Ollama's docs give no indication fine-tuning is supported end-to-end, so (c) would require an entirely separate external toolchain (not Ollama) and is not justified for this use case.
- Context-window size is therefore a real model-selection constraint: 2K (llama2-uncensored) is likely too small to hold a full style profile plus several reference lyric excerpts plus generation instructions; the 32K (dolphin-mistral, dolphin-mixtral:8x7b) to 256K (Qwen3.5-abliterated) tier gives much more headroom.

---

## 6. Alternatives to Ollama

Only one alternative surfaced with any documented signal worth noting: **LM Studio**. However, no primary source found gives LM Studio a documented advantage specific to this project's needs (uncensored models, multilingual creative writing) — it uses the same GGUF model files and the same underlying llama.cpp-family inference, so the same Dolphin/abliterated models run on either. The only sourced differentiators are generic UX ones (LM Studio: Electron GUI, visual parameter tweaking, interactive chat; Ollama: CLI/daemon + REST API, better suited to being embedded inside a custom app's backend).[^lmstudio-search] Since this project is building a custom Windows desktop app (not asking end users to run a separate chat GUI), **Ollama's REST API + Modelfile system is the better architectural fit**, and nothing primary-source-verified suggests switching. Not pursued further.

---

## Recommendation

**Start with `dolphin-mistral:7b`.**

Reasoning, based strictly on what was verified above (not general reputation):

1. **License is unambiguous and fully permissive.** It's built on Mistral's Apache 2.0 base, and Ollama's own model page states outright it is "available for both commercial and non-commercial use" — no attribution notices, no MAU clause, no prohibited-use flow-down to bake into the app's EULA (unlike every Llama- or Gemma-based candidate).
2. **Explicitly uncensored by its own model card**, via the same Hartford/Dolphin dataset-filtering methodology used across the family — not a "reduced guardrails" hedge, and with no Ollama-hosted production-use disclaimer (unlike the `huihui_ai` Qwen3.5-abliterated card, which explicitly cautions against production deployment).
3. **Realistic on a typical consumer Windows PC**: ~4.1 GB at default quantization — runs on modest GPUs or CPU-only with 16 GB RAM, unlike the Mixtral-based Dolphin variants (28–86 GB, needing a 24GB+ VRAM GPU or workstation-class RAM).
4. **32K context window** is comfortably large enough to hold a style profile plus multiple reference-lyric excerpts for the few-shot prompting approach identified in Section 5, versus llama2-uncensored's 2K ceiling.
5. **Multilingual caveat, acknowledged, not hidden:** no primary source found makes an explicit Russian/German fluency claim for Mistral-based Dolphin models — this is a real known-unknown, not a confirmed strength. If Russian/German output quality proves inadequate in testing, the documented fallback is `huihui_ai/qwen3.5-abliterated` (4B or 9B tier), since Qwen3.5 is the only family whose own vendor makes an explicit broad multilingual claim (201 languages) — traded off against its Ollama-hosted production-use caution and an unverifiable license on the specific abliterated checkpoint (HF repo returned 401).
6. **Upgrade path stays in-license:** if output quality/creativity needs to go up and the user has a 24GB+ VRAM GPU, `dolphin-mixtral:8x7b` is a drop-in swap using the identical Apache 2.0 license family and identical uncensoring methodology — no new legal review needed, only a hardware check.

**Explicitly not recommended as the starting default:** anything Llama-3-based (Meta's own card states Llama 3 is out-of-scope for non-English use — directly conflicting with the Russian/German requirement), anything Gemma-based (Google's Gemma Terms impose redistribution/notice obligations on the shipped app), and the `huihui_ai` Qwen3.5-abliterated line as a *first* choice (Ollama's own page discourages production use and the exact checkpoint's license could not be independently verified due to a gated HF repo) — though it remains the best-documented option specifically *if* multilingual quality testing shows Dolphin-family models underperform in Russian/German.

---

## Sources

[^dm]: [ollama.com/library/dolphin-mixtral](https://ollama.com/library/dolphin-mixtral) — model card, sizes, context window, "uncensored" language.
[^dm-tags]: [ollama.com/library/dolphin-mixtral/tags](https://ollama.com/library/dolphin-mixtral/tags) — full quantization size table.
[^dm-hf]: [huggingface.co/cognitivecomputations/dolphin-2.7-mixtral-8x7b](https://huggingface.co/cognitivecomputations/dolphin-2.7-mixtral-8x7b) — Apache 2.0 license, "uncensored... filtered the dataset to remove alignment and bias," English-only language note.
[^dmi]: [ollama.com/library/dolphin-mistral](https://ollama.com/library/dolphin-mistral) — model card: Mistral 0.2 base, "commercial and non-commercial use," 32K context, 4.1GB size.
[^dl3]: [ollama.com/library/dolphin-llama3](https://ollama.com/library/dolphin-llama3) — model card: sizes, context window (8K/256K), "filtered to remove alignment and bias."
[^dl3-hf]: [huggingface.co/cognitivecomputations/dolphin-2.9-llama3-8b](https://huggingface.co/cognitivecomputations/dolphin-2.9-llama3-8b) — Meta Llama 3 Community License confirmation, uncensored statement.
[^l2u]: [ollama.com/library/llama2-uncensored](https://ollama.com/library/llama2-uncensored) — model card: sizes, 2K context, RAM minimums, Hartford methodology reference.
[^wlu]: [ollama.com/library/wizardlm-uncensored](https://ollama.com/library/wizardlm-uncensored) — model card: 13B, 4K context, 16GB RAM minimum, dataset filtering description.
[^search]: [ollama.com/search?q=uncensored](https://ollama.com/search?q=uncensored) — full search result listing (wizard-vicuna-uncensored, everythinglm, etc.).
[^qwen-abl]: [ollama.com/huihui_ai/qwen3.5-abliterated](https://ollama.com/huihui_ai/qwen3.5-abliterated) — model card: sizes, abliteration technique reference, explicit production-use caution.
[^huihui]: [huggingface.co/huihui-ai](https://huggingface.co/huihui-ai) — publisher profile, scope of published models.
[^mixtral-hf]: [huggingface.co/mistralai/Mixtral-8x7B-Instruct-v0.1](https://huggingface.co/mistralai/Mixtral-8x7B-Instruct-v0.1) — Apache 2.0 license confirmation, "5 languages" note.
[^l3-hf]: [huggingface.co/meta-llama/Meta-Llama-3-8B-Instruct](https://huggingface.co/meta-llama/Meta-Llama-3-8B-Instruct) — Meta Llama 3 Community License terms (attribution, MAU clause, output-use restriction), explicit English-only / out-of-scope-for-other-languages statement.
[^qwen-license]: [huggingface.co/Qwen/Qwen3.5-35B-A3B](https://huggingface.co/Qwen/Qwen3.5-35B-A3B) — Apache 2.0 license confirmation, "201 languages and dialects" multilingual claim.
[^gemma-terms]: [ai.google.dev/gemma/terms](https://ai.google.dev/gemma/terms) — Gemma redistribution/notice/prohibited-use terms.
[^modelfile]: [github.com/ollama/ollama/blob/main/docs/modelfile.mdx](https://github.com/ollama/ollama/blob/main/docs/modelfile.mdx) — SYSTEM, PARAMETER num_ctx (default 2048), TEMPLATE, MESSAGE, ADAPTER instructions.
[^ctxlen]: [github.com/ollama/ollama/blob/main/docs/context-length.mdx](https://github.com/ollama/ollama/blob/main/docs/context-length.mdx) — default context sizing by VRAM tier, OLLAMA_CONTEXT_LENGTH override.
[^gpu-doc]: [github.com/ollama/ollama/blob/main/docs/gpu.mdx](https://github.com/ollama/ollama/blob/main/docs/gpu.mdx) — scheduler VRAM behavior, no fixed sizing table published.
[^lmstudio-search]: Web search aggregating LM Studio vs. Ollama comparison sources (secondary; used only to confirm no primary-source-documented advantage exists for this use case — flagged as such in Section 6 rather than treated as authoritative).
