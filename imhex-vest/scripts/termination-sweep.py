#!/usr/bin/env python3
"""Runs the static termination check over a directory of patterns.

    scripts/termination-sweep.py [PATTERNS_DIR] [INCLUDES_DIR]

Defaults: vendor/ImHex-Patterns/patterns and vendor/ImHex-Patterns/includes.
Needs target/release/hexpat (cargo build --release -p hexpat). Prints how
many patterns are fully proven, the obligations by verdict, and the reasons
for the ones that are not proven.
"""
import collections
import os
import re
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
PATTERNS = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "vendor/ImHex-Patterns/patterns")
INCLUDES = sys.argv[2] if len(sys.argv) > 2 else os.path.join(ROOT, "vendor/ImHex-Patterns/includes")
HEXPAT = os.path.join(ROOT, "target/release/hexpat")

LINE = re.compile(r"^(?P<file>[^:]+):(?P<line>\d+): (?P<what>.*?): (?P<tag>proven|run-time check|not proven): (?P<why>.*)$")


def generalize(why):
    """Drops names and numbers so that equal reasons group together."""
    why = re.sub(r"`[^`]*`", "`…`", why)
    why = re.sub(r"\d+ byte\(s\)", "N byte(s)", why)
    why = re.sub(r"by \d+ ", "by N ", why)
    why = re.sub(r"with .* an entry", "with … an entry", why)
    return re.sub(r"uses .*, which", "uses …, which", why)


def main():
    names = sorted(f for f in os.listdir(PATTERNS) if f.endswith(".hexpat"))
    status = collections.Counter()
    verdicts = collections.Counter()
    reasons = collections.Counter()
    sites = set()
    failed = []
    for name in names:
        r = subprocess.run([HEXPAT, "termination", os.path.join(PATTERNS, name), "-I", INCLUDES, "--all"],
                           capture_output=True, text=True, timeout=120)
        if r.returncode not in (0, 2):
            failed.append((name, (r.stderr.strip().splitlines() or ["?"])[-1]))
            continue
        status["proven" if r.returncode == 0 else "open"] += 1
        for line in r.stdout.splitlines():
            m = LINE.match(line)
            if not m:
                continue
            verdicts[m["tag"]] += 1
            # A loop in a library file shows up once per pattern that uses it.
            sites.add((m["file"] if "/" in m["file"] else name, m["line"], m["what"]))
            if m["tag"] != "proven":
                reasons[(m["tag"], generalize(m["why"]))] += 1
    print(f"{len(names)} patterns: {status['proven']} fully proven, {status['open']} with open obligations, "
          f"{len(failed)} not analysed")
    total = sum(verdicts.values())
    print(f"{total} obligations ({len(sites)} distinct sites): {verdicts['proven']} proven, "
          f"{verdicts['run-time check']} need a run-time check, {verdicts['not proven']} not proven")
    for (tag, why), n in reasons.most_common():
        print(f"{n:5}  {tag}: {why}")
    for name, err in failed:
        print(f"failed: {name}: {err}")


if __name__ == "__main__":
    main()
