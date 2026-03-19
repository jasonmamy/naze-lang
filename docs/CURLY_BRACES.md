# Why Curly Braces? Research on LLMs and Significant Whitespace

Naze uses brace-delimited blocks (`{}`) instead of indentation-based syntax. This document collects the research behind that decision.

## TL;DR

- For **fresh code generation** on benchmarks, indentation errors are rare (<0.25%).
- For **code editing** (the agentic workflow Naze targets), whitespace errors are still a real problem across all major models in 2025.
- The strongest argument is **fault tolerance**: wrong indentation in Python = syntax error; wrong indentation in brace-delimited code = cosmetic. Braces make the language resilient to the errors LLMs still make.

## Academic Papers

### Pan et al. (2025) — "The Hidden Cost of Readability: How Code Formatting Silently Consumes Your LLM Budget"

- **URL:** https://arxiv.org/abs/2508.13666
- **Published:** August 2025
- **Models tested:** GPT-3.5-turbo, GPT-4o-mini, GPT-4o, Gemini-1.5, Claude-3.7, Phi-3.5, Qwen-2.5, MagiCoder, DeepSeek-V3, Deepseek-coder-1.3B
- **Key findings:**
  - LLMs maintain performance across formatted and unformatted code — Pass@1 shows negligible change when formatting is removed.
  - Input token reduction when formatting removed: **Java 34.9%, Python only 6.51%** — because Python's whitespace IS the syntax and cannot be removed.
  - Claude-3.7 and GPT-4o demonstrated "remarkable stability" even with formatting removed.
  - **Implication for Naze:** In brace-delimited languages, formatting is redundant (the braces carry structure). In whitespace-sensitive languages, formatting IS syntax and cannot be simplified.

### Wen et al. (2024) — "Fixing Code Generation Errors for Large Language Models"

- **URL:** https://arxiv.org/abs/2409.00676
- **Published:** September 2024
- **Models tested:** 14 LLMs (GPT-3.5-turbo, CodeLlama variants, WizardCoder, Phi series, etc.)
- **Key findings:**
  - IndentationError represented only **0.25% of all errors** (32 out of 12,837 total errors on benchmarks).
  - Root cause: "inconsistent indentation" where indentation levels within the same code block disagree.
  - Their LlmFix method achieves 99.74% fix rate for SyntaxErrors on HumanEval.
  - **Caveat:** Tested on greenfield generation (writing fresh code), not editing existing code.

### Sharifloo et al. (2025) — "Where Do LLMs Still Struggle? An In-Depth Analysis of Code Generation Benchmarks"

- **URL:** https://arxiv.org/abs/2511.04355
- **Published:** November 2025
- **Key findings:**
  - Formatting mistakes are one of four primary failure patterns in code generation.
  - Formatting errors: 0 on HumanEval, 0 on MBPP, 10 on LiveCodeBench, 32 on BCB-Hard.
  - Does not specifically isolate indentation from other formatting issues.

### Xie et al. (2026) — "Rethinking Code Complexity Through the Lens of Large Language Models"

- **URL:** https://arxiv.org/abs/2602.07882
- **Published:** February 2026
- **Key finding:** LLM difficulty arises from "nonlinearity of semantic units" and hierarchical nesting, not surface syntactic choices like braces vs indentation. Suggests structural depth matters more than the delimiter mechanism.

## The Tokenization Problem

The mechanical reason whitespace-sensitive syntax is harder for LLMs:

**GPT-2/3 era:** Each space in Python indentation was a separate token. 4 spaces = 4 tokens, 8 spaces = 8 tokens. This wasted sequence length and made counting difficult.

**GPT-4+ fix:** The cl100k_base tokenizer groups 4 spaces into a single token (token ID 257) and has dedicated tokens for whitespace sequences up to 83-128 consecutive spaces. This was a deliberate engineering fix.

**Why it still matters:** LLMs operate on subword tokens, not characters. A brace `{` or `}` is always a single unambiguous token. Indentation level requires the model to track cumulative whitespace state — even with improved tokenizers, this is an indirect encoding of structure.

Sources:
- Matt Rickard, "The Problems with Tokenization in LLMs" — https://blog.matt-rickard.com/p/the-problems-with-tokenization-in
- RunPod, "Why LLMs Can't Spell Strawberry" — https://www.runpod.io/blog/llm-tokenization-limitations

## Real-World Bug Reports (2025)

Even with modern models, whitespace errors persist in agentic code editing:

### Copilot: "Agent MCP edit tools constantly having indentation issue with Python"

- **URL:** https://github.com/microsoft/vscode-copilot-release/issues/12732
- **Filed:** June 2025, Closed August 2025
- **Models affected:** Claude 3.7, Claude 4, GPT-5, Gemini 2.5 Pro
- **Key quotes:**
  - "constant mis-indentation may occur at function prototypes or docstrings"
  - "a small indentation error would lead to huge amount of time for agent to iterate"
  - Users spend "half my time waiting for it to fix indentation issues that it created"

### Claude Family: Invalid whitespace injection into code syntax

- **URL:** https://github.com/orgs/community/discussions/183048
- **Filed:** December 2025
- **Models affected:** Claude Haiku 4.5, Claude Opus 4.5, Claude Sonnet 4, Claude Sonnet 4.5
- **Examples:** `self. pid` instead of `self.pid`, `os. path. exists` instead of `os.path.exists`
- **Control group:** GPT, Gemini, and Grok models did NOT exhibit this with identical prompts.

### Copilot: Repeated indentation errors in file edits

- **URL:** https://github.com/microsoft/vscode-copilot-release/issues/10255
- Copilot makes Python edits with 2 extra spaces or missing CRLF, then spirals trying to fix its own errors.

### Claude Code: Trailing whitespace consistently added

- **URL:** https://github.com/anthropics/claude-code/issues/363

## The Asymmetry Argument

This is the strongest case for braces in an LLM-targeted language:

| Scenario | Python (whitespace) | Naze (braces) |
|---|---|---|
| LLM gets indentation wrong | **Syntax error** — program does not run | **Cosmetic issue** — program runs fine |
| Error rate on benchmarks | ~0.25% (Wen et al.) | Equivalent or lower |
| Impact of that 0.25% | Fatal | Harmless |

Even if modern LLMs had identical error rates for whitespace and braces, the **consequences** are fundamentally different. Braces make the language fault-tolerant.

## Counterarguments

1. **Benchmark data says indentation errors are rare:** 0.25% error rate (Wen et al.), zero on HumanEval/MBPP (Sharifloo et al.).
2. **Modern tokenizers fixed the mechanical issue:** GPT-4+ treats 4 spaces as one token.
3. **Bug reports may blame tooling, not the model:** The VS Code regression was attributed to Copilot's diff-application layer, not the model's raw output.
4. **Model variation:** GPT-5 reportedly handles Python indentation well; the problem may be model-specific.
5. **Python dominates training data:** LLMs see enormous amounts of Python in pre-training, making them well-calibrated for its indentation patterns.

## Conclusion

The claim "LLMs struggle with significant whitespace" is no longer categorically true for fresh code generation. But for the **agentic editing** workflow Naze is designed for — where models modify existing files — whitespace errors remain a real and documented problem. Braces don't just help LLMs generate correct code; they ensure that when an LLM makes a formatting mistake, the code still runs.
