# Wish List

Ambitious and speculative ideas worth noting. These aren't on the roadmap yet -- they're explorations of what becomes possible as Naze matures.

---

## Voice-Driven Conversational Development

**The idea:** Talk to a local LLM to build and edit Naze apps in real time. No keyboard, no editor -- just a conversation.

### How it works

```
You speak → STT transcribes → local LLM edits .naze files → nazec run hot-reloads → you see the result → TTS reads the LLM's response back to you
```

The loop:

1. You say: "Add a red sidebar with three nav links"
2. Speech-to-text transcribes your words (~200-1200ms depending on engine)
3. A local Naze-trained LLM (3-7B parameters) edits `app.naze`
4. `nazec run` detects the file change, rebuilds in milliseconds, re-renders
5. You see the sidebar appear in the native preview window
6. The LLM responds via text-to-speech: "Added a red sidebar with Home, Settings, and Profile links"
7. You say: "Make it darker and add icons"

### Why Naze makes this tractable

- **Sub-second rebuild.** The hot-reload pipeline (`nazec run`) already rebuilds and re-renders in milliseconds. The bottleneck is the LLM, not the compiler.
- **Constrained grammar.** Naze has one way to express layout, one way to bind data, one way to handle events. A small fine-tuned model (3-7B) on `.naze` could match or outperform a general-purpose 70B model at Naze generation. That model runs locally, offline, at zero cost.
- **Deterministic output.** Same `.naze` source always produces the same render tree. The model can predict exactly what you'll see.
- **Human-readable source.** You can always open the `.naze` file and verify or hand-edit what the model generated.

### Recommended STT/TTS stack

**CPU-only (works on any machine):**

| Component | Library | Latency | Notes |
|-----------|---------|---------|-------|
| STT | [Vosk](https://alphacephei.com/vosk/) | ~1.2s | Lightweight, offline, Rust bindings via vosk-rs, models from 50MB |
| TTS | [Piper](https://github.com/rhasspy/piper) | <500ms | ONNX Runtime, natural-sounding, many voices, runs on CPU |

**With GPU (lower latency):**

| Component | Library | Latency | Notes |
|-----------|---------|---------|-------|
| STT | [Kyutai Moshi](https://github.com/kyutai-labs/moshi) | ~200ms | Streaming, real-time, Rust implementation available |
| TTS | [Kokoro-82M](https://huggingface.co/hexgrad/Kokoro-82M) | <300ms | Tiny model, high quality, Apache 2.0 |

**Voice cloning / expressive TTS:**

| Component | Library | Latency | Notes |
|-----------|---------|---------|-------|
| TTS | [Coqui XTTS-v2](https://github.com/coqui-ai/TTS) | ~1-2s | Voice cloning from a 6-second sample, multilingual, MPL 2.0 |

**Accuracy-first alternative for STT:** [faster-whisper](https://github.com/SYSTRAN/faster-whisper) -- CTranslate2-optimized Whisper, best transcription accuracy, ~2-3s latency on CPU (real-time on GPU).

**Other notable STT options:** [Whisper.cpp](https://github.com/ggerganov/whisper.cpp) -- C/C++ port of OpenAI Whisper, runs on CPU without Python, good for embedding in native toolchains.

### What needs to exist first

- ~~Phase 2 state and events (so generated apps are interactive)~~ Done
- A fine-tuned Naze LLM -- see [LLM.md](LLM.md) for the full training plan, dataset strategy, hardware requirements, and cost estimates (M47 in Phase 6)
- An orchestration layer that pipes STT → LLM → file write → TTS
- ~~The hot-reload pipeline~~ Done (`nazec dev` and `nazec run` both support hot reload)

### Why this matters

Current AI coding tools (Copilot, Cursor, Claude Code) work through text editors. You type, AI suggests, you accept or reject. Voice-driven development removes the editor entirely for the common case. For accessibility, for speed, for the "I'm walking around and want to sketch an app" use case -- this is a different interaction model that Naze's architecture uniquely enables.
