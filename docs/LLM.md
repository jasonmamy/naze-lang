# Fine-Tuning a Local Naze LLM

Train a small model (3-7B parameters) that generates correct `.naze` code better than a general-purpose 70B model. Serve it locally via Ollama for zero-cost, offline, private app generation.

---

## Why a small model can work

General-purpose coding LLMs are large because they cover 50+ languages, hundreds of frameworks, and millions of patterns. Naze is one language with a constrained grammar -- limited keywords, no ambiguous syntax, one way to express each concept. This drastically reduces the search space. A fine-tuned 3B model on `.naze` only needs to learn:

- ~10 element types (`rect`, `text`, `heading`, `row`, `column`, `container`, `stack`, `grid`, `spacer`, `image`)
- ~20 property names (`width`, `height`, `padding`, `gap`, `color`, `radius`, `font-size`, etc.)
- Component definitions and `use` imports
- State, events, conditionals, iteration (Phase 2)

That's orders of magnitude less surface area than "all of React + CSS + HTML + JavaScript."

---

## Training dataset

### Seed corpus (already exists)

The project has 17 `.naze` files in `examples/` that serve as the starting seed:

| File | UI pattern |
|------|-----------|
| `hello.naze` | Welcome screen |
| `boxes.naze` | Colored rectangles with varying radius |
| `columns.naze` | Vertical list/menu |
| `rows.naze` | Horizontal item display |
| `nested.naze` | Card grid with labels |
| `grid.naze` | Thumbnail/icon grid |
| `padding.naze` | Card/panel with internal spacing |
| `rounded.naze` | Corner radius showcase |
| `colors.naze` | Color palette (18 squares in 3 rows) |
| `typography.naze` | Typographic hierarchy |
| `dashboard-static.naze` | Admin dashboard with header, nav sidebar, metric cards |
| `app-shell.naze` | App shell (top nav, sidebar, content, stats) |
| `multi-component.naze` | Component composition |
| `component-basic.naze` | Importing and invoking components |
| `component-props.naze` | Components with typed props |
| `components/color-box.naze` | Component definition: colored square |
| `components/card.naze` | Component definition: styled card with heading + text |

Plus code samples from the README and docs/LANGUAGE.md.

### Tier 1 — AI-generated seeds, human-validated (~200 total)

Expand from 17 to 200 using Claude Code (or similar AI tools) to generate `.naze` files, then validate with a two-step pipeline:

1. **`nazec check`** — automatic syntax and type validation (rejects invalid code immediately)
2. **`nazec run`** — human visually inspects the rendered output in the native preview window (confirms the UI looks correct and intentional)

The human role is curation, not authorship. Claude Code generates candidates covering the full grammar; the human approves or rejects based on visual quality. This makes Tier 1 achievable in a single day rather than multiple days of manual coding.

Target categories:

| Category | Count | What it covers |
|----------|-------|----------------|
| Element showcase | ~30 | Each of the 9 element types with varied property combinations |
| Layout combinations | ~40 | row-in-column, column-in-row, grid-in-container, stack overlays, spacer positioning, nested 3-4 levels deep |
| Component definitions | ~30 | Components with 1-5 params, all 4 param types (text, number, color, bool), default values, nested bodies |
| Component usage | ~30 | Single imports, multiple imports, prop passing, components-using-components |
| Real-world screens | ~50 | Dashboard, login form, card grid, nav bar, settings page, profile page, landing page, pricing table, error/404 page, sidebar menu, footer, modal/dialog, list view, detail view, notification panel, stats overview, file browser, calendar grid, chat layout, checkout form |
| Edge cases | ~20 | Deep nesting (5+ levels), many children (20+ siblings), empty containers, large text blocks, minimal single-element apps, all-spacer layouts |

### Tier 2 — Synthetic expansion via frontier model (~5,000)

Use Claude or GPT-4 to generate training pairs, validated by the compiler:

**Variation generation (~2,000):** For each of the 200 seeds, generate 10 variations -- different colors, sizes, text content, restructured layout. Automated via script that prompts the model with the original file and asks for a variation.

**Intent-to-code pairs (~2,000):** Natural language description → valid `.naze` output:

```
### Instruction
Create a pricing page with three columns: Basic ($9/mo), Pro ($29/mo), Enterprise ($99/mo).
Each card should have a white background, rounded corners, and a colored header.

### Response
app "Pricing" {
  column padding: 20px, gap: 16px {
    heading "Choose a Plan"
    row gap: 16px {
      container padding: 0px, color: #ffffff, radius: 12px, width: 220px {
        column gap: 0px {
          container padding: 16px, color: #3b82f6, radius: 0px {
            heading "Basic" color: #ffffff, font-size: 20px
          }
          column padding: 16px, gap: 8px {
            heading "$9" font-size: 32px
            text "/month" color: #6b7280
          }
        }
      }
      ...
    }
  }
}
```

**Edit instruction pairs (~1,000):** Existing `.naze` + modification request → updated `.naze`:

```
### Existing code
app "Dashboard" {
  column gap: 0px {
    container padding: 16px, color: #1e293b {
      heading "Dashboard" color: #ffffff
    }
    ...
  }
}

### Edit instruction
Change the header background to dark blue (#1e3a5f) and add a subtitle "Welcome back" below the heading.

### Response
app "Dashboard" {
  column gap: 0px {
    container padding: 16px, color: #1e3a5f {
      column gap: 4px {
        heading "Dashboard" color: #ffffff
        text "Welcome back" color: #94a3b8
      }
    }
    ...
  }
}
```

**Quality filter:** Every generated output runs through `nazec check`. Failures are discarded. Expect ~60-80% pass rate from a frontier model on first attempt, yielding ~5,000 valid pairs from ~7,000-8,000 generations.

### Tier 3 — Self-improvement loop (~10,000-20,000)

After the first fine-tune, use the trained model itself to generate more data:

1. Generate 10,000 candidate `.naze` files from varied prompts
2. Filter through `nazec check` -- keep only those that compile
3. Add passing examples to the dataset
4. Fine-tune again on the expanded dataset
5. Repeat 3-5 rounds

**The Apple UICoder precedent proves this works for UI languages.** Apple Research (2024) fine-tuned a model on SwiftUI starting from near-zero Swift training data. Their self-improvement loop (generate → compile → evaluate → filter → retrain) over 5 rounds produced ~996,000 SwiftUI programs and took compilation rate from **3% → 82%** -- matching GPT-4's rate. The researchers explicitly stated this generalizes to "other toolchains with similar properties (e.g., Dart/Flutter, React Native)."

Naze has an even stronger advantage: a simpler, more constrained grammar than SwiftUI, plus a purpose-built compiler that checks types, required props, and component interfaces -- not just syntax.

### Tier 4 — Error correction pairs (~2,000)

Teach the model to fix mistakes by generating (broken code, error, fix) triples:

| Error type | Example | Count |
|------------|---------|-------|
| Wrong prop type | `rect width: "big"` → `rect width: 80px` | ~300 |
| Missing required prop | `rect color: #ff0000` (no width/height) → add dimensions | ~300 |
| Invalid nesting | `text { heading "..." }` → flatten to siblings | ~200 |
| Typos in element names | `colum` → `column`, `containr` → `container` | ~200 |
| Typos in prop names | `paddin:` → `padding:`, `colr:` → `color:` | ~200 |
| Unclosed blocks | Missing `}` → add closing brace | ~200 |
| Invalid color format | `color: red` → `color: #ff0000` | ~200 |
| Unknown props | `rect opacity: 0.5` → remove unsupported prop | ~200 |
| Component misuse | Wrong param types, extra params, missing required params | ~100 |

Script: take valid `.naze` files, programmatically inject errors, capture `nazec check --format json` output, pair with the original.

### Dataset totals

| Tier | Count | Source | Cost |
|------|-------|--------|------|
| Seed corpus | 17 | Already exists | $0 |
| Tier 1: AI-generated, human-validated | ~200 | Claude Code + `nazec run` visual review | ~1 day |
| Tier 2: Synthetic | ~5,000 | Claude/GPT-4 API | ~$20-50 |
| Tier 3: Self-improvement | ~10,000-20,000 | Fine-tuned model + compiler | Compute only |
| Tier 4: Error correction | ~2,000 | Script + compiler | ~$5-10 (API for some) |
| **Total** | **~17,000-27,000** | | **~$25-60** |

---

## 3-week timeline (RTX 4070 Ti SUPER)

Adapted from the bootstrapping plan in [BRAINSTORM.md](BRAINSTORM.md):

```
Week 1: Seed + Expand
  Day 1-2:  Write the Naze grammar in GBNF format for constrained decoding
  Day 2-3:  Test grammar-constrained decoding with a base model → baseline quality
  Day 3-4:  Use Claude Code to generate 200 seed examples (Tier 1)
            Validate each with nazec check, visually review with nazec run
  Day 4-7:  Use Claude/GPT-4 API to expand seeds to 5,000 examples (Tier 2)
            Filter every generated example through nazec check
            Expected yield: ~60-80% pass rate → ~5,000 valid pairs from ~7-8K attempts

Week 2: First Fine-Tune
  Day 8-9:   QLoRA fine-tune Qwen2.5-Coder-3B on the 5K dataset
             RTX 4070 Ti SUPER: ~1-2 hours, ~8-9GB VRAM
  Day 10-11: Evaluate: parse rate, semantic correctness, vs GCD-only baseline
  Day 12-14: Use the fine-tuned model to generate more examples (Tier 3)
             Filter with compiler → expand to 10K-20K examples
             Generate error correction pairs (Tier 4)

Week 3: Iterate
  Day 15-17: Fine-tune on the larger dataset (~17K-27K pairs)
             RTX 4070 Ti SUPER: ~2-4 hours
  Day 18-19: Evaluate, debug failure modes, add targeted examples for weak spots
  Day 20-21: Export to GGUF, create Ollama Modelfile, test end-to-end
             → Production candidate
```

---

## Fine-tuning approach

| Step | Method | Details |
|------|--------|---------|
| Base model | [Qwen2.5-Coder-3B](https://huggingface.co/Qwen/Qwen2.5-Coder-3B) or [CodeLlama-7B](https://huggingface.co/codellama/CodeLlama-7b-hf) | Already trained on code, understands structure and indentation |
| Fine-tune method | QLoRA (4-bit quantized LoRA) | Trains only ~1-2% of parameters, fits on a single consumer GPU |
| Training framework | [Unsloth](https://github.com/unslothai/unsloth) or [Axolotl](https://github.com/OpenAccess-AI-Collective/axolotl) | Simplified fine-tuning wrappers, handle quantization and LoRA setup |
| Merge & quantize | Merge LoRA weights → export GGUF (Q4_K_M or Q5_K_M) | Standard Ollama-compatible format |
| Serve locally | [Ollama](https://ollama.com/) | `ollama create naze-coder -f Modelfile` → `ollama run naze-coder` |

---

## Training on an RTX 4070 Ti SUPER

**Card specs:** 16GB GDDR6X VRAM, Ada Lovelace (AD103), ~44 TFLOPS FP16, 672 GB/s memory bandwidth, 285W TDP.

16GB VRAM is the key constraint. Here's what fits:

| Model size | Method | VRAM usage | Fits? | Training time (10K samples) |
|------------|--------|------------|-------|-----------------------------|
| 3B (Qwen2.5-Coder-3B) | QLoRA 4-bit | ~8-10GB | Yes, comfortably | ~1-2 hours |
| 7B (CodeLlama-7B) | QLoRA 4-bit | ~12-14GB | Yes, tight -- batch size 1-2, gradient checkpointing required | ~4-8 hours |
| 7B | Full fine-tune (FP16) | ~28GB+ | No -- exceeds 16GB | N/A |
| 13B | QLoRA 4-bit | ~16-18GB | No -- just over the limit | N/A |

**Recommended configuration for RTX 4070 Ti SUPER:**

```python
# Unsloth example for Qwen2.5-Coder-3B QLoRA on 16GB VRAM
from unsloth import FastLanguageModel

model, tokenizer = FastLanguageModel.from_pretrained(
    model_name="Qwen/Qwen2.5-Coder-3B",
    max_seq_length=2048,       # .naze files are short, 2048 is plenty
    load_in_4bit=True,         # 4-bit quantization -- essential for 16GB
)

model = FastLanguageModel.get_peft_model(
    model,
    r=16,                      # LoRA rank -- 16 is a good starting point
    lora_alpha=32,
    target_modules=["q_proj", "k_proj", "v_proj", "o_proj",
                    "gate_proj", "up_proj", "down_proj"],
    lora_dropout=0.05,
)

# Training args tuned for 16GB VRAM
training_args = {
    "per_device_train_batch_size": 4,   # 3B fits batch=4 easily
    "gradient_accumulation_steps": 4,   # effective batch size = 16
    "num_train_epochs": 3,
    "learning_rate": 2e-4,
    "fp16": True,                       # Ada Lovelace has good FP16
    "optim": "adamw_8bit",              # 8-bit optimizer saves VRAM
}
```

For the 7B model, reduce `per_device_train_batch_size` to 1, enable `gradient_checkpointing=True`, and reduce `max_seq_length` to 1024 if needed.

**VRAM budget breakdown (3B QLoRA):**

| Component | VRAM |
|-----------|------|
| Base model (4-bit quantized) | ~2GB |
| LoRA adapter weights | ~0.5GB |
| Optimizer states (8-bit AdamW) | ~1GB |
| Activations (batch=4, seq=2048) | ~3-4GB |
| CUDA overhead | ~1GB |
| **Total** | **~8-9GB** |
| **Headroom remaining** | **~7GB** |

That 7GB of headroom means the 3B fine-tune is very comfortable. You could increase batch size to 8 or use a longer sequence length if needed.

**VRAM budget breakdown (7B QLoRA):**

| Component | VRAM |
|-----------|------|
| Base model (4-bit quantized) | ~4GB |
| LoRA adapter weights | ~0.8GB |
| Optimizer states (8-bit AdamW) | ~1.5GB |
| Activations (batch=1, seq=2048, gradient checkpointing) | ~4-5GB |
| CUDA overhead | ~1GB |
| **Total** | **~12-13GB** |
| **Headroom remaining** | **~3GB** |

Tight but workable. Gradient checkpointing trades compute for VRAM -- training will be ~30% slower but fits within 16GB.

**Inference (serving the finished model via Ollama):**

| Model | Quantization | VRAM for inference | Tokens/sec (RTX 4070 Ti SUPER) |
|-------|--------------|--------------------|-------------------------------|
| 3B | Q4_K_M | ~2.5GB | ~100-140 tok/s |
| 7B | Q4_K_M | ~5GB | ~60-90 tok/s |
| 7B | Q5_K_M | ~6GB | ~50-75 tok/s |

A typical `.naze` file is 200-500 tokens, so the 3B model generates a complete file in 2-4 seconds and the 7B in 4-8 seconds. Both leave plenty of VRAM to run STT/TTS models simultaneously.

**Running LLM + STT + TTS simultaneously:**

| Component | VRAM estimate |
|-----------|---------------|
| Naze-coder 3B Q4 inference | ~2.5GB |
| Whisper.cpp medium (STT) | ~2GB |
| Piper (TTS) | ~0.5GB |
| **Total** | **~5GB** |
| **Remaining for desktop/OS** | **~11GB** |

The full voice-driven development stack (LLM + STT + TTS) fits comfortably on this card with room to spare. You could even run the 7B model (~5GB) alongside Whisper and Piper and still have ~8GB free.

---

## Estimated costs

**Self-hosted (RTX 4070 Ti SUPER):**

| Resource | Requirement | Cost |
|----------|-------------|------|
| GPU | RTX 4070 Ti SUPER (16GB) -- 3B comfortably, 7B with care | Already owned |
| Training time | 3B model: ~1-2 hours, 7B model: ~4-8 hours | Electricity only (~$0.10-0.30) |
| Dataset prep | Largest time investment -- writing, generating, validating pairs | Human time + API costs |
| Synthetic data generation | ~5K-10K pairs via Claude/GPT-4 API | ~$20-50 in API calls |

**Cloud GPU rental (if you want faster 7B training or to try 13B+):**

| Provider | GPU | Cost/hour | 7B fine-tune estimate |
|----------|-----|-----------|----------------------|
| RunPod | A100 40GB | ~$1.50/hr | ~$10-20 |
| Lambda Labs | A100 80GB | ~$2.00/hr | ~$12-24 |
| Vast.ai | RTX 4090 | ~$0.40/hr | ~$3-5 |

**Total realistic budget:** ~$25-60 for a first working fine-tune using the RTX 4070 Ti SUPER locally (mostly API costs for synthetic data generation). Cloud rental only needed if you want to experiment with 13B+ models that exceed 16GB VRAM.

---

## Validation loop

The compiler is the validator. Every generated `.naze` file can be checked:

```bash
nazec check --format json < generated.naze
```

This creates a tight feedback loop for training:
- Generate candidate → `nazec check` → if errors, add to "fix this" training pairs
- Measure pass rate: what percentage of generations type-check on first attempt
- Target: >80% first-attempt pass rate for common UI patterns

**Quality at each stage:**

| Stage | Syntax validity | Semantic quality | Useful for |
|-------|----------------|-----------------|------------|
| Grammar-constrained decoding only (no training) | 100% | Moderate -- valid but often incoherent | Autocomplete, suggestions with human review |
| First fine-tune (5K examples) | 100% (with GCD) | 60-80% correct for simple constructs | Boilerplate generation, simple components |
| Iterated fine-tune (20K+ examples) | 100% (with GCD) | 80-90%+ for common patterns | Primary development tool |

---

## Ollama integration

Once the model is exported as GGUF:

```bash
# Create the Ollama model
ollama create naze-coder -f Modelfile

# Modelfile contents:
# FROM ./naze-coder-3b-q4_k_m.gguf
# SYSTEM "You are a Naze UI language expert. Generate valid .naze code."
# PARAMETER temperature 0.3
# PARAMETER num_ctx 4096

# Use it
ollama run naze-coder "Create a settings page with a form"
```

Inference on a 3B Q4 model runs at ~30-60 tokens/sec on a modern CPU, ~100+ tokens/sec with GPU. A typical `.naze` file is 20-50 lines (~200-500 tokens), so generation completes in under 10 seconds on CPU, under 5 seconds on GPU.

---

## Stretch: constrained decoding

Because Naze has a formal grammar (the PEG grammar in `naze-parser`), it's possible to use constrained/guided decoding -- force the model to only output tokens that are valid according to the grammar. Libraries like [Outlines](https://github.com/dottxt-ai/outlines) or [llama.cpp grammars](https://github.com/ggerganov/llama.cpp/blob/master/grammars/README.md) support this. This would push first-attempt pass rate toward 100% at the cost of some generation speed.

The Naze compiler checks more than just syntax -- it validates types, required props, component interfaces, and import resolution. Grammar-constrained decoding handles the syntax layer for free, letting the fine-tuning focus entirely on semantic quality (does the generated code do what the user asked?).
