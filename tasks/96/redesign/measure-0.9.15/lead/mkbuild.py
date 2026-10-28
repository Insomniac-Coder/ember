"""The lead's build and run lists for the measuring phase (tasks/96/redesign/measure-0.9.15).
Reads each job's build.txt and runs.txt and writes:
  build_clang.sh   - every clang and rustc line, rebuilt fresh (the jobs' own lines, variables resolved)
  build_msvc.bat   - the MSVC twin of every C program: cl /O2 /MD /fp:precise /std:c11, named <name>_msvc.exe
  lists/<workload>.tsv - label, exe, args, expected checksum (or '' when only agreement is checked)
"""
import re
import shlex
from pathlib import Path

ROOT = Path(r"C:\Users\ism19\Code\ember")
SRC = ROOT / "tasks/96/redesign/measure-0.9.15"
OUT = ROOT / "target/measure-0.9.15"
JOBS = ["M1", "M2", "M3", "M4"]


def win(p: Path) -> str:
    return str(p).replace("/", "\\")


clang_lines, msvc_lines, c_programs = [], [], {}
for job in JOBS:
    bin_dir = OUT / job / "bin"
    cwd = ROOT if job == "M3" else SRC / job
    for raw in (SRC / job / "build.txt").read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not (line.startswith('"$CC"') or line.startswith('"$CL"') or line.startswith("rustc")):
            continue
        toks = shlex.split(line.replace('"$CC"', "CLANG").replace('"$CL"', "CLANG"))
        out = Path(toks[toks.index("-o") + 1].replace("$B", str(bin_dir)).replace("$O", str(bin_dir)))
        name = out.stem
        srcs = [t.replace("$S", (SRC / "M3").as_posix()) for t in toks if t.endswith(".c") or t.endswith(".rs")]
        defs = [t[2:] for t in toks if t.startswith("-D")]
        if toks[0] == "rustc":
            clang_lines.append(f'(cd "{cwd.as_posix()}" && rustc -C opt-level=2 --edition 2021 -o "{(bin_dir / (name + ".exe")).as_posix()}" {" ".join(srcs)})')
            continue
        if "-c" in toks:  # samevar_size objects: code size only, clang only
            clang_lines.append(f'(cd "{cwd.as_posix()}" && "/c/Program Files/LLVM/bin/clang.exe" ' + " ".join(toks[1:]).replace("$B", bin_dir.as_posix()) + ")")
            continue
        clang_lines.append(f'(cd "{cwd.as_posix()}" && "/c/Program Files/LLVM/bin/clang.exe" -O2 ' + " ".join("-D" + d for d in defs) + f' {" ".join(srcs)} -o "{(bin_dir / (name + ".exe")).as_posix()}")')
        objdir = OUT / job / "obj" / name
        abs_srcs = [win(cwd / s) if not Path(s).is_absolute() else win(Path(s)) for s in srcs]
        msvc_lines.append(f'if not exist "{win(objdir)}" mkdir "{win(objdir)}"')
        msvc_lines.append(f'cl /nologo /O2 /MD /fp:precise /std:c11 ' + " ".join("/D" + d for d in defs) + " " + " ".join(f'"{s}"' for s in abs_srcs)
                          + f' /Fe:"{win(bin_dir / (name + "_msvc.exe"))}" /Fo:"{win(objdir)}\\\\" || echo FAILED {name}')
        c_programs[(job, name)] = True

(OUT / "build_clang.sh").write_text("#!/bin/bash\nset -e\n" + "\n".join(clang_lines) + "\nrm -f " + " ".join(f"{(OUT / j / 'bin').as_posix()}/*.pdb" for j in JOBS) + "\necho CLANG_DONE\n", encoding="utf-8", newline="\n")
(OUT / "build_msvc.bat").write_text("@echo off\r\ncall \"C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Auxiliary\\Build\\vcvars64.bat\" >nul 2>nul\r\n"
                                     + "\r\n".join(msvc_lines) + "\r\necho MSVC_DONE\r\n", encoding="utf-8", newline="\r\n")

# run lists, one per workload
lists = {}


def add(wl, job, exe, args, expected):
    name = exe[:-4]
    b = OUT / job / "bin"
    tag = (" " + args) if args else ""
    lists.setdefault(wl, []).append((name + tag, win(b / exe), args, expected))
    if (job, name) in c_programs:
        lists[wl].append((name + "_msvc" + tag, win(b / (name + "_msvc.exe")), args, expected))


for line in (SRC / "M1/runs.txt").read_text(encoding="utf-8").splitlines():
    if line.startswith("#") or not line.strip():
        continue
    exe, _, ck = line.split("\t")
    wl = next(w for w in ("retarget", "hotshare", "weakview", "selfremove") if w in exe)
    add(wl, "M1", exe, "", ck.strip())
for line in (SRC / "M2/runs.txt").read_text(encoding="utf-8").splitlines():
    if line.startswith("#") or not line.strip():
        continue
    exe = line.split()[0]
    ck = line.split()[-1]
    wl = "w2v" if "w2v" in exe else "tree"
    add(wl, "M2", exe, "", ck if ck.isdigit() else "")
for line in (SRC / "M3/runs.txt").read_text(encoding="utf-8").splitlines():
    if line.startswith("#") or not line.strip():
        continue
    exe, args, w = line.split()
    add({"W1": "trees", "W2": "lists", "W3": "churn"}[w], "M3", exe, "" if args == "-" else args, "")
wl = None
for line in (SRC / "M4/runs.txt").read_text(encoding="utf-8").splitlines():
    m = re.match(r"# (\w+)(?:,[^:]*)?:", line)
    if m:
        wl = m.group(1)
        continue
    if line.startswith("#") or not line.strip():
        continue
    exe, args, ck = [x.strip() for x in line.split("|")]
    add(wl, "M4", exe, args, ck)

(OUT / "lists").mkdir(exist_ok=True)
for wl, rows in lists.items():
    (OUT / "lists" / f"{wl}.tsv").write_text("\n".join("\t".join(r) for r in rows) + "\n", encoding="utf-8")
print(len(clang_lines), "clang/rustc lines;", len(msvc_lines) // 2, "msvc programs;", {k: len(v) for k, v in lists.items()})
