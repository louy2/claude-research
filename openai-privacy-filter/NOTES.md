# Testing OpenAI Privacy Filter

Evaluating https://github.com/openai/privacy-filter (released Apr 22, 2026, Apache 2.0,
1.5B params, 50M active, 128k context) on a public PII dataset.

## Status

Blocked: HuggingFace was not in the sandbox network allowlist; weights live at
`openai/privacy-filter` on HF. Added `huggingface.co` / `hf.co` to
`~/.claude/settings.json` `sandbox.network.allowedDomains`. Change requires a
session restart before `pip`/`hf_hub_download` can reach the host.

## Reproduce (after HF allowlist takes effect)

```bash
# 1. Install (already done in this workspace)
# From this research directory (openai-privacy-filter/):
git clone --depth 1 https://github.com/openai/privacy-filter.git work/privacy-filter
pip install --break-system-packages torch
pip install --break-system-packages huggingface_hub safetensors tiktoken datasets
pip install --break-system-packages -e work/privacy-filter

# 2. Smoke test (downloads ~3GB to ~/.opf/privacy_filter on first run)
opf --device cpu "Alice Smith lives at 123 Main St; alice@x.com; (415) 555-0123."

# 3. Evaluate on a public PII dataset (script TBD)
python scripts/build_eval_jsonl.py --dataset ai4privacy/pii-masking-300k \
    --split validation --limit 500 --out work/eval_300k.jsonl
opf eval work/eval_300k.jsonl --device cpu --eval-mode untyped
```

## Dataset choices

- `ai4privacy/pii-masking-300k` — benchmark OpenAI reports F1=96% on; validates
  their numbers against a held-out split.
- `ai4privacy/pii-masking-200k` — predecessor, likely out-of-distribution if
  OpenAI trained only on 300k.
- A code/log dataset (e.g. `bigcode/bigcode-pii-dataset`) for cross-domain check.

## OPF taxonomy (8 categories)

`account_number`, `private_address`, `private_email`, `private_person`,
`private_phone`, `private_url`, `private_date`, `secret`.

Eval modes: `typed` (category-level F1, only when dataset uses OPF labels) or
`untyped` (span-level F1, works across taxonomies). For ai4privacy → OPF, use
`untyped`; otherwise build a label map and use `typed`.

## Eval JSONL format

```json
{"text": "...", "spans": {"private_person: Alice": [[0,5]], ...}, "info": {"id": "..."}}
```

Spans are character offsets `[start, end)` keyed by `"<label>: <surface form>"`.
