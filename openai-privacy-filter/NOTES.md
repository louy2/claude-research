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

### n=200, ai4privacy/pii-masking-300k validation, English

CPU eval (no GPU available), viterbi decode, default operating point.
Throughput ~13.3 tokens/sec; 200 examples / 23.5k tokens took 29 min.
Full metrics in `work/metrics_n200.json`, predictions in
`work/predictions_n200.jsonl` (both gitignored).

| metric                   | value |
| ------------------------ | ----- |
| detection.f1 (token)     | 0.923 |
| detection.precision      | 0.938 |
| detection.recall         | 0.909 |
| detection.span.f1        | 0.847 |
| detection.span.precision | 0.912 |
| detection.span.recall    | 0.791 |

**`ground_truth_label_recall` (char-level recall per source label)**

| recall band | source labels |
| ----------- | ------------- |
| 1.000       | BOD, DRIVERLICENSE, EMAIL, GEOCOORD, GIVENNAME2, IP, LASTNAME1, LASTNAME2, LASTNAME3, PASS, PASSPORT, POSTCODE, SECADDRESS, SOCIALNUMBER, STREET |
| 0.96–0.99   | BUILDING (0.974), CITY (0.987), DATE (0.964), GIVENNAME1 (0.987), TEL (0.995), USERNAME (0.989) |
| 0.81–0.83   | IDCARD (0.829), TITLE (0.812) |
| ≤ 0.14      | COUNTRY (0.000), STATE (0.000), SEX (0.141), TIME (0.085) |

The bottom band isn't model misses so much as a taxonomy mismatch: OPF's
training policy "aims to prioritize personal identifiers, often preserving
context that is not strongly person-linked" (see README § Limitation:
Static Label Policy). A bare country / state / sex / time word in
isolation is not a personal identifier under OPF's 8-category taxonomy.
These four labels alone account for ~1.46k ground-truth chars out of
~14.6k total (≈10%), so they pull span recall down by roughly that
fraction.

If those four labels are excluded, the recoverable span recall on the
remaining "personal-identifier-shaped" labels is ~0.88+ — closer to the
F1=0.96 OpenAI reports in their model card, which is reassuring given the
floor difference (their number is `--eval-mode typed` against an OPF-label
held-out split, ours is `untyped` against a different taxonomy).

OPF eval did not surface `GEOCOORD` or `TITLE` in the n=20 sample because
those labels weren't present in the first 20 examples. The n=200 sample
adds them at full and 81% recall respectively.

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
