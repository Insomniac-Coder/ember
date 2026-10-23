#!/usr/bin/env python3
"""`[TOOL-2]` — for the release workflow only: download the pinned LLVM and Ninja for one host
(`tools/toolchain-pins.json`), check each against its recorded SHA-256, and unpack them for
`tools/release_archive.py`:

    python tools/fetch_toolchain.py <host> <dir>
        -> <dir>/llvm/ (bin/, lib/clang/<major>/include/), <dir>/ninja/ninja[.exe]

A file whose hash is not the recorded one is refused; one with no hash recorded yet is refused too,
with the hash found printed, so the first release records it only once someone has checked it
against the project's own release page.
"""

import hashlib
import json
import shutil
import sys
import tarfile
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def fetch(url, target):
    with urllib.request.urlopen(url) as response, open(target, "wb") as out:
        shutil.copyfileobj(response, out)


def checked(name, entry, file):
    digest = hashlib.sha256(file.read_bytes()).hexdigest()
    if not entry["sha256"]:
        sys.exit(f"{name}: no SHA-256 is recorded in tools/toolchain-pins.json for {entry['url']}; it downloaded as {digest}. "
                 "Check it against the project's release page and record it.")
    if digest != entry["sha256"]:
        sys.exit(f"{name}: {entry['url']} is {digest}, not the recorded {entry['sha256']}: refused")


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    host, out = sys.argv[1], Path(sys.argv[2])
    pins = json.loads((ROOT / "tools" / "toolchain-pins.json").read_text(encoding="utf-8"))
    out.mkdir(parents=True, exist_ok=True)
    downloads = out / "downloads"
    downloads.mkdir(exist_ok=True)

    llvm = pins["llvm"][host]
    archive = downloads / llvm["url"].rsplit("/", 1)[1]
    fetch(llvm["url"], archive)
    checked("LLVM", llvm, archive)
    unpacked = out / "llvm-unpacked"
    shutil.rmtree(unpacked, ignore_errors=True)
    with tarfile.open(archive) as tar:
        tar.extractall(unpacked, filter="data")
    roots = [path for path in unpacked.iterdir() if path.is_dir()]
    root = roots[0] if len(roots) == 1 else unpacked
    shutil.rmtree(out / "llvm", ignore_errors=True)
    shutil.move(str(root), out / "llvm")

    ninja = pins["ninja"][host]
    archive = downloads / ninja["url"].rsplit("/", 1)[1]
    fetch(ninja["url"], archive)
    checked("Ninja", ninja, archive)
    with zipfile.ZipFile(archive) as zip_file:
        zip_file.extractall(out / "ninja")
    executable = out / "ninja" / ("ninja.exe" if host == "windows-x64" else "ninja")
    executable.chmod(0o755)
    print(f"LLVM {pins['llvm']['version']} in {out / 'llvm'}; Ninja {pins['ninja']['version']} at {executable}")


if __name__ == "__main__":
    main()
