#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Activate venv if it exists
if [ -f ai/.venv/bin/activate ]; then
  source ai/.venv/bin/activate
fi

echo "=== Preflight checks ==="

# Check Ollama is installed
if ! command -v ollama &>/dev/null; then
  echo "Error: ollama not found. Install from https://ollama.com" >&2
  exit 1
fi
echo "  ollama: $(ollama --version)"

# Check Ollama is running
if ! ollama list &>/dev/null; then
  echo "Error: ollama is not running. Start it with: ollama serve" >&2
  exit 1
fi
echo "  ollama server: running"

# Check a model is available for dataset export (description generation)
EXPORT_MODEL="${OLLAMA_MODEL:-qwen2.5-coder:7b}"
if ! ollama list | grep -q "${EXPORT_MODEL%%:*}"; then
  echo "Error: model '${EXPORT_MODEL}' not found. Pull it with: ollama pull ${EXPORT_MODEL}" >&2
  echo "  Or set OLLAMA_MODEL to use a different model." >&2
  exit 1
fi
echo "  export model: ${EXPORT_MODEL}"

echo "=== Step 1: Build nazec ==="
cargo build -p nazec --release

echo "=== Step 1.5: Generate additional examples ==="
python3 ai/generate_examples.py

echo "=== Step 2: Export examples as JSONL ==="
./target/release/nazec ai dataset export \
  --dir examples \
  --provider ollama \
  --model "${EXPORT_MODEL}" \
  --output ai/data/raw_export.jsonl

echo "=== Step 3: Validate raw dataset ==="
./target/release/nazec ai dataset validate ai/data/raw_export.jsonl

echo "=== Step 4: Prepare ChatML training data ==="
cd ai
python3 prepare_dataset.py
cd ..

echo "=== Step 5: Fine-tune ==="
echo "  Unloading all Ollama models to free VRAM..."
for m in $(ollama ps 2>/dev/null | tail -n +2 | awk '{print $1}'); do
  echo "  Stopping $m..."
  ollama stop "$m" 2>/dev/null || true
done
sleep 3
cd ai
python3 train.py
cd ..

echo "=== Step 6: Register with Ollama ==="
cd ai
ollama create naze-coder -f Modelfile
cd ..

echo "=== Step 7: Smoke test ==="
./target/release/nazec ai generate \
  --provider ollama --model naze-coder \
  "Create a counter app with increment and reset buttons"

echo ""
echo "=== Done! ==="
echo "Use: nazec ai generate --provider ollama --model naze-coder \"<description>\""
