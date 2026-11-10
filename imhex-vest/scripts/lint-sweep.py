#!/usr/bin/env python3
"""Runs `hexpat lint` over a directory of patterns and counts the findings.

    scripts/lint-sweep.py [--list CHECK] [PATTERNS_DIR] [INCLUDES_DIR]

Defaults: vendor/ImHex-Patterns/patterns and vendor/ImHex-Patterns/includes.
Needs target/release/hexpat. `--list CHECK` prints every finding of one
check (for example `--list duplicate-member`).
"""
import collections
import os
import re
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
HEXPAT = os.path.join(ROOT, "target/release/hexpat")
FINDING = re.compile(r"^(?P<where>[^ ]+): (?P<level>warning|note)\[(?P<check>[a-z-]+)\]: (?P<message>.*)$")


def main():
    args = sys.argv[1:]
    listed = None
    if args[:1] == ["--list"]:
        listed, args = args[1], args[2:]
    patterns = args[0] if args else os.path.join(ROOT, "vendor/ImHex-Patterns/patterns")
    includes = args[1] if len(args) > 1 else os.path.join(ROOT, "vendor/ImHex-Patterns/includes")
    names = sorted(f for f in os.listdir(patterns) if f.endswith(".hexpat"))
    counts = collections.Counter()
    with_check = collections.defaultdict(set)
    clean = failed = 0
    for name in names:
        r = subprocess.run([HEXPAT, "lint", os.path.join(patterns, name), "-I", includes],
                           capture_output=True, text=True, timeout=120)
        if r.returncode not in (0, 2):
            failed += 1
            continue
        clean += r.returncode == 0
        for line in r.stdout.splitlines():
            m = FINDING.match(line)
            if not m:
                continue
            key = (m["level"], m["check"])
            counts[key] += 1
            with_check[key].add(name)
            if m["check"] == listed:
                print(line)
    print(f"{len(names)} patterns: {clean} without warnings, {len(names) - clean - failed} with warnings, {failed} not analysed")
    for (level, check), n in sorted(counts.items(), key=lambda kv: (kv[0][0] != "warning", -kv[1])):
        print(f"{n:5}  {level}[{check}] in {len(with_check[(level, check)])} pattern(s)")


if __name__ == "__main__":
    main()
