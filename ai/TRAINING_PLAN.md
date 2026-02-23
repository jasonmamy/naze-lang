# Training Data Expansion Plan

## Goal

Scale from ~94 raw examples to ~500 for reliable fine-tuned code generation.

## Why 500?

Research on QLoRA fine-tuning for domain-specific code generation shows:

| Raw Examples | After Augmentation | Quality Level |
|---|---|---|
| 50-100 | ~400 | Fragile — works for trained patterns, fails on novel prompts |
| **200-500** | **~1,700** | **Reliable — handles most prompts within the language's scope** |
| 500-1,000 | ~3,500 | Robust — graceful degradation on edge cases |

Our current 94 raw examples (expanded to ~400 via augmentation) sit at the bottom of the fragile range. Target: **500 raw examples → ~1,700 training samples**.

## Coverage Gaps (Current 94 Examples)

### Features with 0 examples
prompt, boundary/catch, meta tags, stack layout, code element, import, context-menu event, pointer-move event, validate pattern, data:js source, session storage, server fn with fetch/SQL/update, flow tests

### Features with 1-2 examples
template/slot/fill, guard, param, function, server functions, drag events, notify, emit, shared state, debounce/throttle

### Over-represented features (50+ examples)
state, text, heading, column, row, rect, click handler, if/else

## Generation Strategy

### `ai/generate_examples.py`

Programmatic template composition (no LLM needed):

| Category | Count | Method |
|---|---|---|
| Missing features | 30 | Hand-crafted templates for zero-coverage features |
| Feature combinations | 200 | Systematic cross-feature pairing (20 groups x 10 variants) |
| Complexity variations | 100 | Simple/medium/complex of 10 common patterns |
| App archetypes | 70 | Real-world app skeletons (dashboard, chat, CRM, etc.) |
| **Total** | **~400** | |

### Validation

Every generated file is validated with `nazec parse`. Invalid files are dropped and logged.

### Augmentation Pipeline (existing)

`ai/prepare_dataset.py` applies to ALL examples (existing + generated):

| Augmentation | Multiplier | Method |
|---|---|---|
| Paraphrase variants | 2x | Template-based instruction rewording (5 verb templates) |
| Error-fix pairs | 0.5x | Deliberate syntax errors → model learns to fix |
| Explanation pairs | 0.4x | Reverse direction (code → description) |

### Expected Dataset Size

```
  94 existing examples
+ 400 generated examples
= 494 raw examples

+ 10 curated few-shot
+ ~1,000 paraphrase variants (2 per example)
+ ~250 error-fix pairs (50% of examples)
+ ~200 explanation pairs (40% of examples)
= ~1,700 total training samples
```

## Output

Generated files go to `examples/generated/` (separate from hand-crafted examples). The `nazec ai dataset export --dir examples` command recurses into subdirectories automatically.

## Running

```bash
# Test batch (10 examples)
python3 ai/generate_examples.py --limit 10

# Full generation
python3 ai/generate_examples.py

# Full pipeline (includes generation)
bash ai/run_pipeline.sh
```
