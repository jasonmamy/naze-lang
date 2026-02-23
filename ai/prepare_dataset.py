#!/usr/bin/env python3
"""Convert raw JSONL from `nazec ai dataset export` into ChatML training data.

Input:  ai/data/raw_export.jsonl  ({"instruction": ..., "response": ...} per line)
Output: ai/data/train.jsonl, ai/data/eval.jsonl  ({"messages": [...]} per line)

Augmentations:
  - Error-fix pairs: introduce a deliberate error, train model to fix it
  - Explanation pairs: reverse direction (code → description)
"""

import json
import random
import re
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
DATA_DIR = SCRIPT_DIR / "data"
RAW_FILE = DATA_DIR / "raw_export.jsonl"
TRAIN_FILE = DATA_DIR / "train.jsonl"
EVAL_FILE = DATA_DIR / "eval.jsonl"
AGENTS_MD = PROJECT_ROOT / "docs" / "AGENTS_SLIM.md"

SEED = 42

# ─── Instruction paraphrase templates ────────────────────────────────────────
# Template-based rewording to multiply training pairs without needing new code.
# {desc} is replaced with the original instruction (lowercased, stripped of
# leading verbs like "Create", "Build", etc.)

INSTRUCTION_TEMPLATES = [
    "Create {desc}",
    "Build {desc}",
    "Make {desc}",
    "Write Naze code for {desc}",
    "Generate {desc}",
]

# Verbs to strip from the start of instructions before re-templating
_STRIP_VERBS = [
    "create ", "build ", "make ", "write ", "generate ", "implement ",
    "design ", "define ", "add ", "set up ", "develop ",
    "write naze code for ",
]


def paraphrase_instruction(instruction: str, count: int = 2) -> list[str]:
    """Generate `count` paraphrased variants of an instruction."""
    lower = instruction.strip().lower()
    desc = lower
    for verb in sorted(_STRIP_VERBS, key=len, reverse=True):
        if desc.startswith(verb):
            desc = desc[len(verb):]
            break

    # Pick `count` templates that differ from the original
    variants = []
    for tmpl in INSTRUCTION_TEMPLATES:
        candidate = tmpl.format(desc=desc)
        # Skip if it's too similar to the original (case-insensitive)
        if candidate.lower().strip() == lower:
            continue
        variants.append(candidate)
        if len(variants) >= count:
            break
    return variants


# ─── Curated examples (match the 10 embedded in ai.rs) ───────────────────────
# These have hand-written descriptions that don't depend on an LLM.
CURATED_EXAMPLES = [
    {
        "instruction": "Create a counter app with increment and reset buttons",
        "response": """-- Counter with increment and reset
app "Counter" {
  state count = 0
  column padding: 20px, gap: 16px {
    heading "My Counter"
    text "Count: {count}"
    rect width: 200px, height: 50px, color: #2563eb, radius: 8px {
      text "Increment"
      on click: set count = count + 1
    }
    rect width: 200px, height: 50px, color: #dc2626, radius: 8px {
      text "Reset"
      on click: set count = 0
    }
  }
}""",
    },
    {
        "instruction": "Build a dashboard layout with header, sidebar, and metric cards",
        "response": """-- Dashboard layout with header, sidebar, and metric cards
app "Dashboard" {
  column gap: 0px {
    container padding: 16px, color: #1e293b {
      heading "Dashboard" font-size: 20px, color: #ffffff
    }
    row padding: 20px, gap: 20px {
      column width: 200px, gap: 8px, padding: 16px, color: #f8fafc {
        text "Overview"
        text "Analytics"
        text "Settings"
      }
      column gap: 16px {
        heading "Overview"
        row gap: 16px {
          container padding: 16px, color: #eff6ff, radius: 8px, width: 180px {
            column gap: 4px {
              text "Revenue"
              heading "$12,345" font-size: 24px
            }
          }
          container padding: 16px, color: #f0fdf4, radius: 8px, width: 180px {
            column gap: 4px {
              text "Users"
              heading "1,234" font-size: 24px
            }
          }
        }
      }
    }
  }
}""",
    },
    {
        "instruction": "Create an app that fetches data from an API with loading and error states",
        "response": """-- Data fetching with loading/error states
app "Posts" {
  data posts: fetch "https://jsonplaceholder.typicode.com/posts?_limit=5"

  column gap: 16px, padding: 20px {
    heading "API Data"

    if posts.loading {
      text "Loading...", color: #666666
    }

    if posts.error {
      text "Error: {posts.error}", color: #dc2626
    }

    if posts.data {
      each post in posts.data {
        column padding: 12px, color: #f3f4f6, radius: 8px {
          heading "{post.title}", font-size: 16px
          text "{post.body}", color: #666666, font-size: 14px
        }
      }
    }
  }
}""",
    },
    {
        "instruction": "Build an app using pipeline operators to filter, sort, and aggregate data",
        "response": """-- Pipeline operators: filter, sort, aggregate
app "Student Scores" {
  state students = [{name: "Alice", score: 92}, {name: "Bob", score: 67}, {name: "Carol", score: 85}, {name: "Dave", score: 45}]

  computed passing = students | filter score > 60
  computed total-score = students | map score | sum
  computed student-count = students | count

  column padding: 20px, gap: 16px {
    heading "Student Scores"
    text "Total students: {student-count}"
    text "Total score: {total-score}"

    heading "Passing (score > 60):" font-size: 18px
    each student in students | filter score > 60 | sort-by name {
      text "{student.name}: {student.score}"
    }
  }
}""",
    },
    {
        "instruction": "Create a sign-up form with validation and error display",
        "response": """-- Form with validation and error display
app "Sign Up" {
  state username = ""
  state email = ""

  column padding: 20px, gap: 12px {
    heading "Create Account"

    text "Username"
    input bind: username, placeholder: "Enter username", validate: { required: true, min-length: 3, max-length: 20 }
    if username_error {
      text "{username_error}" color: #dc2626
    }

    text "Email"
    input bind: email, type: "email", placeholder: "Enter email", validate: { required: true }
    if email_error {
      text "{email_error}" color: #dc2626
    }

    row gap: 8px {
      if username_valid {
        text "Username OK" color: #16a34a
      }
      if email_valid {
        text "Email OK" color: #16a34a
      }
    }
  }
}""",
    },
    {
        "instruction": "Create a multi-page app with navigation between Home and About pages",
        "response": """-- Multi-page app with navigation
app "My Site" {
  row padding: 16px, gap: 24px, color: #1e293b {
    heading "My App" color: #ffffff, font-size: 18px
    link "Home", to: "/"
    link "About", to: "/about"
  }

  page "/" {
    column padding: 24px, gap: 16px {
      heading "Welcome Home"
      text "Click the links above to navigate."
    }
  }

  page "/about" {
    column padding: 24px, gap: 16px {
      heading "About Us"
      text "Built with Naze."
    }
  }
}""",
    },
    {
        "instruction": "Define a reusable card component with typed parameters and defaults",
        "response": """-- Component definition with typed parameters and defaults
component card(bg: color = #ffffff, width: number = 200px) {
  container padding: 16px, color: bg, radius: 8px, width: width {
    column gap: 8px {
      heading "Card Title" font-size: 16px
      text "Card content goes here."
    }
  }
}""",
    },
    {
        "instruction": "Create an app using pattern matching to show different statuses",
        "response": """-- Pattern matching for conditional rendering
app "Match Demo" {
  state status = "active"

  column padding: 20px, gap: 16px {
    heading "Pattern Matching"

    match status {
      "active": text "Status: Active" color: #16a34a
      "inactive": text "Status: Inactive" color: #dc2626
      "pending": text "Status: Pending..." color: #eab308
      _: text "Status: Unknown"
    }

    row gap: 8px {
      rect width: 80px, height: 36px, color: #16a34a, radius: 4px {
        text "Active" color: #ffffff
        on click: set status = "active"
      }
      rect width: 80px, height: 36px, color: #dc2626, radius: 4px {
        text "Inactive" color: #ffffff
        on click: set status = "inactive"
      }
    }
  }
}""",
    },
    {
        "instruction": "Build an app with light and dark theme switching",
        "response": """-- Named themes with extends and runtime switching
theme light {
  colors {
    bg: #ffffff
    fg: #0f172a
    primary: #2563eb
  }
  spacing {
    sm: 8px
    md: 16px
  }
}

theme dark extends light {
  colors {
    bg: #1e293b
    fg: #f8fafc
    primary: #60a5fa
  }
}

app "Theme Demo" {
  column padding: 20px, gap: 16px, color: theme.colors.bg {
    heading "Theme Switching" color: theme.colors.fg
    row gap: 12px {
      rect width: 100px, height: 40px, color: theme.colors.primary, radius: 8px {
        text "Light" color: #ffffff
        on click: set-theme "light"
      }
      rect width: 100px, height: 40px, color: theme.colors.primary, radius: 8px {
        text "Dark" color: #ffffff
        on click: set-theme "dark"
      }
    }
  }
}""",
    },
    {
        "instruction": "Create an app with expand/collapse animation using transitions",
        "response": """-- Transitions and interactive toggle
app "Animation" {
  state expanded = false

  column gap: 16px, padding: 20px {
    heading "Animation Demo"

    if expanded {
      row width: 200px, height: 150px, color: #3b82f6, radius: 8px, padding: 16px, transition: "height 300ms ease-out" {
        text "Click to shrink" color: #ffffff
        on click: set expanded = false
      }
    }
    if expanded == false {
      row width: 200px, height: 60px, color: #3b82f6, radius: 8px, padding: 16px, transition: "height 300ms ease-out" {
        text "Click to expand" color: #ffffff
        on click: set expanded = true
      }
    }
  }
}""",
    },
]


# ─── Error injection for error-fix augmentation ──────────────────────────────

def inject_error(code: str) -> tuple[str, str]:
    """Introduce a deliberate error into Naze code. Returns (broken_code, error_description)."""
    strategies = []

    # Strategy 1: Remove a closing brace
    brace_positions = [i for i, c in enumerate(code) if c == "}"]
    if len(brace_positions) > 1:
        strategies.append("remove_brace")

    # Strategy 2: Misspell a keyword
    keywords = [
        ("column", "colum"),
        ("heading", "headng"),
        ("container", "contaner"),
        ("padding", "paddng"),
        ("computed", "compued"),
    ]
    for correct, wrong in keywords:
        if correct in code:
            strategies.append(f"misspell_{correct}")

    # Strategy 3: Remove a quote
    if '"' in code:
        strategies.append("remove_quote")

    if not strategies:
        return code, ""

    strategy = random.choice(strategies)

    if strategy == "remove_brace":
        # Remove the last closing brace (common beginner mistake)
        pos = brace_positions[-1]
        broken = code[:pos] + code[pos + 1 :]
        return broken, "missing closing brace"

    if strategy == "remove_quote":
        # Find string literals and remove a closing quote
        quote_positions = [i for i, c in enumerate(code) if c == '"']
        if len(quote_positions) >= 2:
            pos = quote_positions[-1]
            broken = code[:pos] + code[pos + 1 :]
            return broken, "missing closing quote"

    if strategy.startswith("misspell_"):
        keyword = strategy.replace("misspell_", "")
        for correct, wrong in keywords:
            if correct == keyword:
                broken = code.replace(correct, wrong, 1)
                return broken, f"misspelled '{correct}' as '{wrong}'"

    return code, ""


# ─── Main pipeline ────────────────────────────────────────────────────────────

def main():
    random.seed(SEED)

    # Load system prompt from AGENTS.md
    if not AGENTS_MD.exists():
        print(f"Error: {AGENTS_MD} not found", file=sys.stderr)
        sys.exit(1)
    system_prompt = AGENTS_MD.read_text().strip()

    # Load raw export
    if not RAW_FILE.exists():
        print(f"Error: {RAW_FILE} not found", file=sys.stderr)
        print("Run: nazec ai dataset export --dir examples --provider ollama --output ai/data/raw_export.jsonl", file=sys.stderr)
        sys.exit(1)

    raw_pairs = []
    with open(RAW_FILE) as f:
        for line in f:
            line = line.strip()
            if line:
                raw_pairs.append(json.loads(line))

    print(f"Loaded {len(raw_pairs)} pairs from raw export")

    # Add curated examples (deduplicate by first line of response)
    raw_first_lines = {p["response"].strip().split("\n")[0] for p in raw_pairs}
    curated_added = 0
    for ex in CURATED_EXAMPLES:
        first_line = ex["response"].strip().split("\n")[0]
        if first_line not in raw_first_lines:
            raw_pairs.append(ex)
            curated_added += 1
    print(f"Added {curated_added} curated examples (deduped)")

    # Build ChatML messages
    all_samples = []

    for pair in raw_pairs:
        instruction = pair["instruction"].strip()
        response = pair["response"].strip()

        # Standard generation pair
        all_samples.append(make_chatml(system_prompt, instruction, f"```naze\n{response}\n```"))

    # Augmentation: paraphrase variants (2 per example)
    paraphrase_count = 0
    for pair in raw_pairs:
        instruction = pair["instruction"].strip()
        response = pair["response"].strip()
        for variant in paraphrase_instruction(instruction, count=2):
            all_samples.append(make_chatml(system_prompt, variant, f"```naze\n{response}\n```"))
            paraphrase_count += 1

    # Augmentation: error-fix pairs (~50% of examples)
    error_fix_count = 0
    for pair in raw_pairs:
        if random.random() > 0.5:
            continue
        response = pair["response"].strip()
        broken, error_desc = inject_error(response)
        if not error_desc:
            continue
        prompt = f"Fix this Naze code (error: {error_desc}):\n\n```naze\n{broken}\n```"
        all_samples.append(make_chatml(system_prompt, prompt, f"```naze\n{response}\n```"))
        error_fix_count += 1

    # Augmentation: explanation pairs (reverse direction, ~40% of examples)
    explanation_count = 0
    for pair in raw_pairs:
        if random.random() > 0.4:
            continue
        instruction = pair["instruction"].strip()
        response = pair["response"].strip()
        prompt = f"Explain what this Naze code does:\n\n```naze\n{response}\n```"
        all_samples.append(make_chatml(system_prompt, prompt, instruction))
        explanation_count += 1

    print(f"Augmented: {paraphrase_count} paraphrase variants, {error_fix_count} error-fix pairs, {explanation_count} explanation pairs")
    print(f"Total samples: {len(all_samples)}")

    # Shuffle and split 90/10
    random.shuffle(all_samples)
    split_idx = int(len(all_samples) * 0.9)
    train_data = all_samples[:split_idx]
    eval_data = all_samples[split_idx:]

    # Write output
    write_jsonl(TRAIN_FILE, train_data)
    write_jsonl(EVAL_FILE, eval_data)

    print(f"Train: {len(train_data)} samples → {TRAIN_FILE}")
    print(f"Eval:  {len(eval_data)} samples → {EVAL_FILE}")


def make_chatml(system: str, user: str, assistant: str) -> dict:
    return {
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
            {"role": "assistant", "content": assistant},
        ]
    }


def write_jsonl(path: Path, data: list[dict]):
    with open(path, "w") as f:
        for item in data:
            f.write(json.dumps(item) + "\n")


if __name__ == "__main__":
    main()
