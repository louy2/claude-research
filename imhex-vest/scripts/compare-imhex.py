#!/usr/bin/env python3
"""Runs ImHex's pattern-language CLI and our hexpat on each corpus pattern's
first fixture (the one hexpat/tests/corpus.rs uses) and compares the JSON.

    scripts/compare-imhex.py CORPUS HEXPAT OUT_DIR

CORPUS is vendor/ImHex-Patterns, HEXPAT the hexpat binary, OUT_DIR a new
directory for both outputs. Prints one tab-separated row per pattern and a
summary on stderr.

ImHex comes from a release flatpak, unpacked without installing it:

    curl -LO https://github.com/WerWolv/ImHex/releases/download/nightly/imhex-1.39.0.WIP-x86_64.flatpak
    ostree init --repo=repo --mode=archive
    flatpak build-import-bundle repo imhex-*.flatpak
    ostree checkout --repo=repo -U app/net.werwolv.ImHex/x86_64/master out
    ln -sfn "$PWD/out/files" /app      # the binary looks for its files there

ImHex writes its logs and settings under HOME, set to a scratch directory
below (IMHEX_HOME overrides it).
"""
import os, subprocess, sys, collections
corpus, hexpat, out = sys.argv[1:4]
os.makedirs(out, exist_ok=False)
td = os.path.join(corpus, "tests/patterns/test_data")
home = os.environ.get("IMHEX_HOME", os.path.join(out, "imhex-home"))
env = dict(os.environ, HOME=home, LD_LIBRARY_PATH="/app/lib:/app/lib64")
entries = sorted(os.listdir(td))
res = collections.Counter()
rows = []
for name in sorted(os.listdir(os.path.join(corpus, "patterns"))):
    if not name.endswith(".hexpat"):
        continue
    fx = []
    for e in entries:
        p = os.path.join(td, e)
        if e == name and os.path.isdir(p):
            fx += [os.path.join(p, x) for x in os.listdir(p) if os.path.isfile(os.path.join(p, x))]
        elif e.startswith(name + ".") and os.path.isfile(p):
            fx.append(p)
    if not fx:
        continue
    fixture = sorted(fx)[0]
    pat = os.path.join(corpus, "patterns", name)
    a = os.path.join(out, name + ".imhex.json")
    b = os.path.join(out, name + ".ours.json")
    try:
        r = subprocess.run(["/app/bin/imhex", "--pl", "format", "-f", "json", "-I", os.path.join(corpus, "includes"),
                            "-p", pat, "-i", fixture, "-o", a], cwd="/tmp", env=env, capture_output=True, text=True,
                           errors="replace", timeout=180)
        ic = r.returncode
        imsg = (r.stdout + r.stderr).replace("sh: 1: zenity: not found", "").replace("sh: 1: notify-send: not found", "").strip()
    except subprocess.TimeoutExpired:
        ic, imsg = "timeout", ""
    try:
        r2 = subprocess.run([hexpat, "format", "-f", "json", "-I", os.path.join(corpus, "includes"), "-p", pat, "-i", fixture,
                             "-o", b], capture_output=True, text=True, errors="replace", timeout=180)
        oc = r2.returncode
    except subprocess.TimeoutExpired:
        oc = "timeout"
    ia, ob = os.path.exists(a), os.path.exists(b)
    if ic == 0 and ia and oc == 0 and ob:
        state = "both ok, identical" if open(a, "rb").read() == open(b, "rb").read() else "both ok, differ"
    elif ic == 0 and ia:
        state = "imhex ok, ours failed"
    elif oc == 0 and ob:
        state = f"imhex failed ({ic}), ours ok"
    else:
        state = f"both failed (imhex {ic})"
    res[state] += 1
    first = imsg.splitlines()[0][:110] if imsg else ""
    for line in imsg.splitlines():
        if "error:" in line.lower():
            first = line.strip()[:110]
            break
    rows.append((name, state, first))
for r in rows:
    print("\t".join(r))
print("---", file=sys.stderr)
for k, v in res.most_common():
    print(v, k, file=sys.stderr)
