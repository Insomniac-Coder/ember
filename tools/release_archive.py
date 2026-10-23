#!/usr/bin/env python3
"""`[TOOL-1]`, `[TOOL-2]` (ODR-222, ODR-223, ODR-258) — assemble one host's Ember toolchain archive.

The archive is self-contained: `ember`, the runtime's sources, `std`, and the C toolchain it builds
with — LLVM's Clang and its linker LLD, and Ninja — which the compiler finds beside itself before
anything on `PATH`. Its layout:

    ember/
      bin/ember[.exe]
      std/                      the standard library (`ember_branding::std_root` walks up to it)
      runtime/ember_rt/         the runtime's headers and sources (`runtime_dir` walks up to it)
      toolchain/llvm/bin/       clang (and clang-cl on Windows), LLD (lld-link, ld.lld or ld64.lld)
      toolchain/llvm/lib/clang/<major>/include/   Clang's own headers
      toolchain/ninja/ninja[.exe]
      toolchain.json            what is carried: versions, and each file's SHA-256
      VERSION

This script only assembles: it reads a built compiler and an LLVM and a Ninja already on disk (the
release workflow downloads the pinned ones; a local run is pointed at installed ones) and writes the
archive and its checksum under `target/release-archive/`. It downloads nothing and uploads nothing.

    python tools/release_archive.py --llvm <LLVM root> --ninja <ninja executable>
        [--ember target/release/ember(.exe)] [--version v0.1.0] [--host windows-x64] [--out DIR]

The archive is reproducible (`[BLD-13]`): its entries are sorted, and each has the same fixed time
and permissions whatever the files' own.
"""

import argparse
import hashlib
import io
import json
import os
import platform
import shutil
import stat
import sys
import tarfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOSTS = ("windows-x64", "linux-x64", "macos-arm64")
# A fixed time for every entry (1980-01-01, the earliest a zip can hold).
FIXED_TIME = (1980, 1, 1, 0, 0, 0)
FIXED_EPOCH = 315532800


def host_now():
    system = platform.system()
    machine = platform.machine().lower()
    if system == "Windows":
        return "windows-x64"
    if system == "Linux" and machine in ("x86_64", "amd64"):
        return "linux-x64"
    if system == "Darwin" and machine in ("arm64", "aarch64"):
        return "macos-arm64"
    sys.exit(f"no Ember archive is made for this host ({system} {machine}); hosts: {', '.join(HOSTS)}")


def cargo_version():
    for line in (ROOT / "Cargo.toml").read_text(encoding="utf-8").splitlines():
        if line.startswith("version = "):
            return "v" + line.split('"')[1]
    sys.exit("no workspace version in Cargo.toml")


def sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as file:
        for block in iter(lambda: file.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def copy_tree(source, target, skip=()):
    for path in sorted(source.rglob("*")):
        relative = path.relative_to(source)
        if any(part in skip for part in relative.parts) or path.is_dir():
            continue
        destination = target / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, destination)


def executable(name, host):
    return name + ".exe" if host == "windows-x64" else name


def llvm_files(llvm, host):
    """What of an LLVM installation the archive carries: the C driver, the linker, Clang's headers."""
    bin_dir = llvm / "bin"
    linker = {"windows-x64": "lld-link", "linux-x64": "ld.lld", "macos-arm64": "ld64.lld"}[host]
    names = ["clang", "clang-cl", linker, "lld"] if host == "windows-x64" else ["clang", linker, "lld"]
    files = []
    for name in names:
        path = bin_dir / executable(name, host)
        if path.is_file():
            files.append((path, Path("toolchain/llvm/bin") / path.name))
    carried = {destination.name for _, destination in files}
    for needed in [executable("clang", host), executable(linker, host)]:
        if needed not in carried:
            sys.exit(f"`{llvm}` has no bin/{needed}: point --llvm at an LLVM installation")
    # Clang's resource directory: `lib/clang/<major>/include`.
    resource = sorted((llvm / "lib" / "clang").glob("*/include"))
    if not resource:
        sys.exit(f"`{llvm}` has no lib/clang/<version>/include (Clang's own headers)")
    include = resource[-1]
    for path in sorted(include.rglob("*")):
        if path.is_file():
            files.append((path, Path("toolchain/llvm/lib/clang") / include.parent.name / "include" / path.relative_to(include)))
    return files, include.parent.name


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--llvm", required=True, type=Path, help="an LLVM installation's root (bin/, lib/clang/)")
    parser.add_argument("--ninja", required=True, type=Path, help="the ninja executable")
    parser.add_argument("--ember", type=Path, help="the built compiler (default target/release/ember[.exe])")
    parser.add_argument("--version", help="the release's tag (default: the workspace version)")
    parser.add_argument("--host", choices=HOSTS, help="the host the archive is for (default: this one)")
    parser.add_argument("--out", type=Path, default=ROOT / "target" / "release-archive")
    args = parser.parse_args()
    host = args.host or host_now()
    version = args.version or cargo_version()
    ember = args.ember or ROOT / "target" / "release" / executable("ember", host)
    if not ember.is_file():
        sys.exit(f"no compiler at `{ember}`: build it first (cargo build --release -p ember_driver --bin ember)")
    if not args.ninja.is_file():
        sys.exit(f"no ninja at `{args.ninja}`")

    stage = args.out / f"ember-{host}"
    shutil.rmtree(stage, ignore_errors=True)
    (stage / "bin").mkdir(parents=True)
    shutil.copyfile(ember, stage / "bin" / executable("ember", host))
    copy_tree(ROOT / "std", stage / "std")
    copy_tree(ROOT / "runtime" / "ember_rt", stage / "runtime" / "ember_rt", skip={"templates", "tests"})
    llvm, clang_major = llvm_files(args.llvm, host)
    for source, destination in llvm:
        (stage / destination).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, stage / destination)
    (stage / "toolchain" / "ninja").mkdir(parents=True)
    shutil.copyfile(args.ninja, stage / "toolchain" / "ninja" / executable("ninja", host))
    for name in ("LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "README.md"):
        if (ROOT / name).is_file():
            shutil.copyfile(ROOT / name, stage / name)
    for name in ("LICENSE.TXT",):
        if (args.llvm / name).is_file():
            shutil.copyfile(args.llvm / name, stage / "toolchain" / "llvm" / name)
    (stage / "VERSION").write_text(version + "\n", encoding="utf-8")

    carried = sorted(path for path in stage.rglob("*") if path.is_file())
    manifest = {
        "version": version,
        "host": host,
        "clang": clang_major,
        "files": {path.relative_to(stage).as_posix(): sha256(path) for path in carried},
    }
    (stage / "toolchain.json").write_text(json.dumps(manifest, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    carried = sorted(path for path in stage.rglob("*") if path.is_file())

    # Entries under `ember/`, sorted, each with the fixed time; executables 0755, the rest 0644.
    def is_executable(path):
        relative = path.relative_to(stage).as_posix()
        return relative.startswith(("bin/", "toolchain/llvm/bin/", "toolchain/ninja/"))

    if host == "windows-x64":
        archive = args.out / f"ember-{host}.zip"
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as out:
            for path in carried:
                info = zipfile.ZipInfo("ember/" + path.relative_to(stage).as_posix(), date_time=FIXED_TIME)
                info.external_attr = ((0o755 if is_executable(path) else 0o644) | stat.S_IFREG) << 16
                info.compress_type = zipfile.ZIP_DEFLATED
                out.writestr(info, path.read_bytes())
    else:
        archive = args.out / f"ember-{host}.tar.gz"
        raw = io.BytesIO()
        with tarfile.open(fileobj=raw, mode="w", format=tarfile.PAX_FORMAT) as out:
            for path in carried:
                info = tarfile.TarInfo("ember/" + path.relative_to(stage).as_posix())
                data = path.read_bytes()
                info.size = len(data)
                info.mtime = FIXED_EPOCH
                info.mode = 0o755 if is_executable(path) else 0o644
                info.uid = info.gid = 0
                info.uname = info.gname = ""
                out.addfile(info, io.BytesIO(data))
        import gzip
        with open(archive, "wb") as file, gzip.GzipFile(filename="", mode="wb", fileobj=file, mtime=FIXED_EPOCH, compresslevel=9) as gz:
            gz.write(raw.getvalue())
    checksum = f"{sha256(archive)}  {archive.name}\n"
    (args.out / f"{archive.name}.sha256").write_text(checksum, encoding="utf-8")
    print(f"{archive} ({archive.stat().st_size // (1 << 20)} MiB)")
    print(checksum, end="")


if __name__ == "__main__":
    main()
