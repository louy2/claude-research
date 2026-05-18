"""Convert ai4privacy/pii-masking-300k examples into OPF eval JSONL.

OPF eval format (one JSON per line):
    {"text": "...",
     "spans": {"<label>: <surface>": [[start, end], ...], ...},
     "info": {"id": "..."}}

Spans are character offsets [start, end) into `text`. We keep the dataset's
original labels (USERNAME, EMAIL, ...); the eval will run in --eval-mode untyped
since these don't match the OPF taxonomy.
"""

import argparse
import json
from collections import defaultdict

from datasets import load_dataset


def build(dataset_name: str, split: str, limit: int, language: str, out_path: str) -> int:
    ds = load_dataset(dataset_name, split=split, streaming=True)
    written = 0
    with open(out_path, "w") as f:
        for ex in ds:
            if language and ex.get("language") != language:
                continue
            text = ex["source_text"]
            spans = defaultdict(list)
            for m in ex["privacy_mask"]:
                start, end, label, value = m["start"], m["end"], m["label"], m["value"]
                # Sanity: the recorded slice should equal value.
                if text[start:end] != value:
                    # Skip malformed entries rather than crashing.
                    continue
                key = f"{label}: {value}"
                spans[key].append([start, end])
            if not spans:
                continue
            row = {"text": text, "spans": dict(spans), "info": {"id": ex["id"]}}
            f.write(json.dumps(row, ensure_ascii=False) + "\n")
            written += 1
            if limit and written >= limit:
                break
    return written


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--dataset", default="ai4privacy/pii-masking-300k")
    p.add_argument("--split", default="validation")
    p.add_argument("--limit", type=int, default=500)
    p.add_argument("--language", default="English")
    p.add_argument("--out", required=True)
    args = p.parse_args()
    n = build(args.dataset, args.split, args.limit, args.language, args.out)
    print(f"wrote {n} examples to {args.out}")


if __name__ == "__main__":
    main()
