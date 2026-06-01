#!/usr/bin/env python3
"""Extract the Swift Windows Platform SDK out of a WiX 'burn' bundle installer.

Usage: _burn_extract.py <burn-manifest> <attached-container.cab> <out-dir>

The burn manifest (the file named "0" inside the UX container, extracted with
`7z x installer.exe`) lists the payloads packed into the attached container, in
order. cabextract enumerates those members as a0, a1, a2, ... in that same
order, so we can map member -> real filename by index and pull out exactly the
MSIs / cabs we need to reconstruct the x86_64 Windows SDK.
"""
import re
import subprocess
import sys

manifest, attached_cab, out_dir = sys.argv[1], sys.argv[2], sys.argv[3]

xml = open(manifest, "rb").read().decode("utf-8", "replace")
# Payloads packed in the attached container, in document order == cab order.
names = [
    m.group(1)
    for p in re.findall(r"<Payload\b[^>]*?/>", xml)
    if 'Container="WixAttachedContainer"' in p
    for m in [re.search(r'FilePath="([^"]*)"', p)]
    if m
]

# windows.msi (the Platform SDK installer) references windows.cab (base),
# sdk.windows.arm64.cab (media 2) and sdk.windows.x64.cab (media 3). msiextract
# walks media sequentially, so media 1..3 must all be present to reach x64.
wanted = {"windows.msi", "windows.cab", "sdk.windows.arm64.cab", "sdk.windows.x64.cab"}
members = {names[i]: f"a{i}" for i in range(len(names)) if names[i] in wanted}
if set(members) != wanted:
    sys.exit(f"manifest missing payloads: {wanted - set(members)}")

args = ["cabextract", "-q", "-d", out_dir]
for member in members.values():
    args += ["-F", member]
subprocess.run([*args, attached_cab], check=True)

import os
for real, member in members.items():
    os.replace(os.path.join(out_dir, member), os.path.join(out_dir, real))
    print("extracted", real)
