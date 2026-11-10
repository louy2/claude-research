#!/usr/bin/env python3
"""Checks the termination verdicts against mutated inputs.

    scripts/termination-fuzz.py [--rounds N] [--max-size BYTES] [PATTERN...]

For every corpus pattern with a test fixture, classifies the pattern with
`hexpat termination` and evaluates it on mutated copies of the fixture
(byte flips, truncation, runs of 0x00 / 0xFF, random data). A run that ends
at one of the interpreter's limits, or that does not end within the
timeout, is reported together with the static verdict:

  proven / guarded   the static check says no run needs a limit, so a limit
                     hit here contradicts it (or the input is simply large)
  open               the static check does not claim anything

Needs target/release/hexpat and vendor/ImHex-Patterns.
"""
import argparse
import os
import random
import subprocess
import sys
import tempfile

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
PATTERNS = os.path.join(ROOT, "vendor/ImHex-Patterns/patterns")
INCLUDES = os.path.join(ROOT, "vendor/ImHex-Patterns/includes")
DATA = os.path.join(ROOT, "vendor/ImHex-Patterns/tests/patterns/test_data")
HEXPAT = os.path.join(ROOT, "target/release/hexpat")
LIMITS = ("limit exceeded", "limit reached", "exceeded the iteration limit")


def fixture(name):
    """The first test file of a pattern: `<name>.hexpat`, `<name>.hexpat.<ext>`,
    or the first file in a directory of that name."""
    for entry in sorted(os.listdir(DATA)):
        if entry == name + ".hexpat" or entry.startswith(name + ".hexpat."):
            p = os.path.join(DATA, entry)
            if os.path.isdir(p):
                files = sorted(os.listdir(p))
                return os.path.join(p, files[0]) if files else None
            return p
    return None


def mutate(rng, data):
    b = bytearray(data)
    kind = rng.randrange(6)
    if kind == 0 and b:  # flip a few bytes
        for _ in range(rng.randint(1, 8)):
            b[rng.randrange(len(b))] = rng.randrange(256)
    elif kind == 1 and b:  # truncate
        del b[rng.randrange(len(b)):]
    elif kind == 2 and b:  # a run of zeros
        at = rng.randrange(len(b))
        n = rng.randint(1, 64)
        b[at:at + n] = bytes(min(n, len(b) - at))
    elif kind == 3 and b:  # a run of 0xFF
        at = rng.randrange(len(b))
        n = rng.randint(1, 64)
        b[at:at + n] = b"\xff" * min(n, len(b) - at)
    elif kind == 4 and b:  # keep the head, randomize the rest
        at = rng.randrange(min(len(b), 64) + 1)
        b[at:] = bytes(rng.randrange(256) for _ in range(len(b) - at))
    else:  # small integers everywhere: offsets and counts that stay in range
        b = bytearray(rng.choice((0, 0, 1, 2, 4, 8, rng.randrange(256))) for _ in range(max(len(b), 16)))
    return bytes(b)


def verdict(pattern):
    r = subprocess.run([HEXPAT, "termination", pattern, "-I", INCLUDES], capture_output=True, text=True, timeout=120)
    return {0: "proven", 3: "guarded", 2: "open"}.get(r.returncode, "error")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rounds", type=int, default=20)
    ap.add_argument("--max-size", type=int, default=65536)
    ap.add_argument("--timeout", type=float, default=20)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--keep", help="directory to copy the inputs of reported runs to")
    ap.add_argument("patterns", nargs="*")
    args = ap.parse_args()
    rng = random.Random(args.seed)
    names = args.patterns or sorted(f[:-7] for f in os.listdir(PATTERNS) if f.endswith(".hexpat"))
    runs = hits = 0
    by_verdict = {}
    with tempfile.TemporaryDirectory() as tmp:
        path = os.path.join(tmp, "input.bin")
        for name in names:
            fx = fixture(name)
            pattern = os.path.join(PATTERNS, name + ".hexpat")
            if fx is None or not os.path.isfile(pattern):
                continue
            data = open(fx, "rb").read()[:args.max_size]
            v = verdict(pattern)
            by_verdict[v] = by_verdict.get(v, 0) + 1
            seen = set()
            for i in range(args.rounds):
                with open(path, "wb") as f:
                    f.write(mutate(rng, data))
                runs += 1
                try:
                    r = subprocess.run([HEXPAT, "run", pattern, path, "-I", INCLUDES, "--format"],
                                       capture_output=True, text=True, errors="replace", timeout=args.timeout)
                    out = r.stderr + r.stdout
                    hit = next((l.strip() for l in out.splitlines() if any(k in l for k in LIMITS)), None)
                    if r.returncode < 0:
                        hit = f"killed by signal {-r.returncode}"
                except subprocess.TimeoutExpired:
                    hit = f"no result after {args.timeout:g} s"
                if hit and hit not in seen:
                    seen.add(hit)
                    hits += 1
                    if args.keep:
                        os.makedirs(args.keep, exist_ok=True)
                        with open(os.path.join(args.keep, f"{name}-{i}.bin"), "wb") as f:
                            f.write(open(path, "rb").read())
                    print(f"{name} [{v}] round {i}: {hit[:160]}", flush=True)
    print(f"{runs} runs over {sum(by_verdict.values())} patterns {by_verdict}; {hits} distinct limit hits or timeouts")


if __name__ == "__main__":
    main()
