# Testing OpenAI Privacy Filter

Evaluating https://github.com/openai/privacy-filter (released Apr 22, 2026, Apache 2.0,
1.5B params, 50M active, 128k context) on a public PII dataset.

## Reproduce

```bash
# From this research directory (openai-privacy-filter/):
git clone --depth 1 https://github.com/openai/privacy-filter.git work/privacy-filter
pip install --break-system-packages --index-url https://download.pytorch.org/whl/cpu torch
pip install --break-system-packages huggingface_hub safetensors tiktoken datasets numpy
pip install --break-system-packages -e work/privacy-filter

# Smoke test (downloads ~3GB to ~/.opf/privacy_filter on first run; CPU run takes a few seconds once loaded)
opf --device cpu "Alice Smith lives at 123 Main St; alice@x.com; (415) 555-0123."
# -> "<PRIVATE_PERSON> lives at <PRIVATE_ADDRESS>; <PRIVATE_EMAIL>; <PRIVATE_PHONE>."

# Build a 500-example eval set from ai4privacy/pii-masking-300k validation split
python3 scripts/build_eval_jsonl.py --limit 500 --out work/eval_300k_val_500.jsonl

# Run eval. ai4privacy labels (USERNAME, EMAIL, ...) don't match the OPF
# taxonomy, so use --eval-mode untyped (span-level matching).
opf eval work/eval_300k_val_500.jsonl --device cpu --eval-mode untyped \
    --max-examples 200 --metrics-out work/metrics_n200.json
```

## Results

### Smoke test (n=20, ai4privacy/pii-masking-300k validation, English)

CPU eval, viterbi decode, default operating point, ~13 tokens/sec on this box.

| metric                  | value |
| ----------------------- | ----- |
| detection.span.f1       | 0.895 |
| detection.span.precision | 0.963 |
| detection.span.recall   | 0.835 |
| detection.f1 (token)    | 0.956 |
| token_accuracy          | 0.785 |

**`ground_truth_label_recall` (char-level recall per source label)**

100% recalled: `BOD`, `BUILDING`, `CITY`, `DATE`, `DRIVERLICENSE`, `EMAIL`,
`GIVENNAME1`, `IDCARD`, `IP`, `LASTNAME1`, `LASTNAME2`, `LASTNAME3`, `PASS`,
`PASSPORT`, `POSTCODE`, `SECADDRESS`, `SOCIALNUMBER`, `STREET`, `TEL`.
~99% recalled: `USERNAME`.
~0% recalled: `COUNTRY`, `SEX`, `STATE`, `TIME` (6.5%).

The four near-zero categories aren't model misses so much as a taxonomy
mismatch: OPF's training policy "aims to prioritize personal identifiers,
often preserving context that is not strongly person-linked" (see README §
Limitation: Static Label Policy). A bare country/state/sex/time word in
isolation is not a personal identifier under OPF's taxonomy. They drag
untyped span F1 down by ~5-6 points.

**Per-class span precision (predicted OPF labels with ≥1 hit)**

`private_address`, `private_person`, `private_phone`, `private_url`: 1.000;
`account_number`: 0.983; `private_date`: 0.917; `private_email`: 0.778;
`secret`: 0.000 (no `secret` ground-truth in this sample, all preds were FP).

n=200 run pending (results in `work/metrics_n200.json` once finished).

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
