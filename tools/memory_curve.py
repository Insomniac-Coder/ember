"""ODR-281 — the compiler's peak memory against project size: it holds only what the module it is
compiling needs, with a small summary of each module it has finished, so its peak is set by the
biggest module, not by the whole project. The proof is the curve.

For each size, a generated `bench/bigpkg` package (`[BUD-1]`'s shape: modules of 125 lines, 15 %
generic code, the Vulkan-sized C header) is built clean (`ember build --timings=json`, the program
cache off, a fresh output folder), then edited and built again `--edits` times (a one-function-body
edit, a leaf's signature and the widely imported type in turn, each build incremental: `[BLD-1]`'s
kept checked forms). Each build's peak memory (`peak_memory_mb`: the compiler's own peak working set,
or resident high-water mark) and time are recorded; per size the clean builds' medians and the
edits' highest peak and median time are published, with the slope of a least-squares line through
the peaks (megabytes per 1,000 lines) and the largest size's peak over the smallest's.

`--before EXE` measures a second compiler beside the first (`--ember`, the after one) on its own
copy of each package, the two interleaved (each round runs both, the first of them in turn, so what
the machine does meanwhile falls on both alike); each size's time is then published as the after
compiler's over the before one's.

`--fanout N` (the default, 64) generates the package whose `main` reaches the modules through a tree
of modules calling at most N each (`generate.py --fanout`): its biggest module is the same at every
size, which is what a flat curve needs; `--fanout 0` keeps bigpkg's own `main`, which calls every
module and so grows with the package (2 lines per module).

The curve is flat when the after compiler's peak at the largest size is within `--flat` (default
10 %, room for the summaries of finished modules) of its peak at the smallest, after the clean build
and after the edits alike; otherwise it is published as growing, with its slope.

    python tools/memory_curve.py [--ember EXE] [--before EXE] [--sizes 50000,100000,200000,400000,800000]
        [--fanout N] [--edits N] [--runs N] [--jobs N] [--work DIR] [--keep] [--flat F]
        [--json FILE] [--markdown FILE]

Exit status: 0 the curve is flat; 1 it grows; 2 a build failed.
"""
import argparse
import importlib.util
import json
import os
import shutil
import statistics
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIGPKG = os.path.join(ROOT, "bench", "bigpkg")
CLI = "ember"
EXE = ".exe" if os.name == "nt" else ""
NO_COMPILE_CACHE_VAR = "EMBER_NO_COMPILE_CACHE"
LINES_PER_MODULE = 125  # [BUD-1]'s 50k lines in 400 modules
EDIT_KINDS = ("body", "signature", "type")


def load_generator():
    spec = importlib.util.spec_from_file_location("bigpkg_generate", os.path.join(BIGPKG, "generate.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class BuildFailed(Exception):
    pass


def build(ember, pkg, out, jobs):
    """One `ember build --timings=json`: its peak memory (MB), its time (s) and each stage's report."""
    env = dict(os.environ, **{NO_COMPILE_CACHE_VAR: "1"})
    command = [ember, "build", "--timings=json", "--out-dir", out] + (["-j", str(jobs)] if jobs else [])
    done = subprocess.run(command, cwd=pkg, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace")
    report = None
    for line in done.stderr.splitlines():
        if line.startswith("{") and "peak_memory_mb" in line:
            report = json.loads(line)
    if done.returncode != 0 or report is None or report.get("peak_memory_mb") is None:
        raise BuildFailed(f"`{CLI} build` failed in {pkg} (exit {done.returncode}):\n{done.stdout[-2000:]}{done.stderr[-2000:]}")
    return {"peak_mb": report["peak_memory_mb"], "seconds": report["total_ms"] / 1000, "timings": report["timings"]}


def fit(points):
    """The least-squares line through (lines, MB): its slope per 1,000 lines and its intercept."""
    xs = [lines / 1000 for lines, _ in points]
    ys = [mb for _, mb in points]
    if len(points) < 2:
        return 0.0, ys[0] if ys else 0.0
    mx, my = statistics.fmean(xs), statistics.fmean(ys)
    sxx = sum((x - mx) ** 2 for x in xs)
    slope = sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / sxx if sxx else 0.0
    return slope, my - slope * mx


def measure(sides, sizes, fanout, edits, runs, jobs, work, keep, log=print):
    """`sides`: {name: compiler}. For each size, each side's clean builds and edit builds, the
    sides interleaved; per size and side the published figures."""
    generator = load_generator()
    rows = []
    order = list(sides)
    for lines in sizes:
        modules = max(lines // LINES_PER_MODULE, 3)
        pkgs = {side: os.path.join(work, f"{side}-pkg-{lines}") for side in sides}
        outs = {side: os.path.join(work, f"{side}-out-{lines}") for side in sides}
        for side in sides:
            shutil.rmtree(pkgs[side], ignore_errors=True)
            generator.generate(pkgs[side], modules, lines, fanout=fanout or None)
        written = sum(open(os.path.join(pkgs[order[0]], "src", name), encoding="utf-8").read().count("\n")
                      for name in os.listdir(os.path.join(pkgs[order[0]], "src")) if name.endswith(".em"))
        clean = {side: [] for side in sides}
        for run in range(runs):
            for side in (order if run % 2 == 0 else order[::-1]):
                shutil.rmtree(outs[side], ignore_errors=True)
                clean[side].append(build(sides[side], pkgs[side], outs[side], jobs))
                log(f"{lines} lines ({modules} modules), {side}: clean build {run + 1}: {clean[side][-1]['peak_mb']:.1f} MB, {clean[side][-1]['seconds']:.1f} s")
        edited = {side: [] for side in sides}
        counts = {kind: 0 for kind in EDIT_KINDS}
        for n in range(edits):
            kind = EDIT_KINDS[n % len(EDIT_KINDS)]
            counts[kind] += 1
            for side in (order if n % 2 == 0 else order[::-1]):
                generator.edit(pkgs[side], kind, counts[kind], modules, lines)
                edited[side].append(dict(build(sides[side], pkgs[side], outs[side], jobs), edit=kind))
                log(f"{lines} lines, {side}: edit {n + 1} ({kind}): {edited[side][-1]['peak_mb']:.1f} MB, {edited[side][-1]['seconds']:.1f} s")
        row = {"lines": written, "modules": modules}
        for side in sides:
            builds = clean[side]
            middle = sorted(builds, key=lambda b: b["peak_mb"])[len(builds) // 2]
            row[side] = {
                "clean": {"peak_mb": statistics.median(b["peak_mb"] for b in builds), "seconds": statistics.median(b["seconds"] for b in builds),
                          "runs": [{"peak_mb": b["peak_mb"], "seconds": b["seconds"]} for b in builds], "stages": middle["timings"]},
                "edits": {"peak_mb": max((b["peak_mb"] for b in edited[side]), default=None),
                          "seconds": statistics.median(b["seconds"] for b in edited[side]) if edited[side] else None,
                          "runs": [{"edit": b["edit"], "peak_mb": b["peak_mb"], "seconds": b["seconds"]} for b in edited[side]]},
            }
        rows.append(row)
        if not keep:
            for side in sides:
                shutil.rmtree(pkgs[side], ignore_errors=True)
                shutil.rmtree(outs[side], ignore_errors=True)
    return rows


def judge(rows, side, flat):
    """One side's curve: flat when the largest size's peak is within `flat` of the smallest's, after
    the clean build and after the edits alike."""
    result = {}
    for kind in ("clean", "edits"):
        points = [(row["lines"], row[side][kind]["peak_mb"]) for row in rows if row[side][kind]["peak_mb"] is not None]
        if not points:
            continue
        slope, intercept = fit(points)
        growth = points[-1][1] / points[0][1] if points[0][1] else 1.0
        result[kind] = {"slope_mb_per_1000_lines": slope, "intercept_mb": intercept,
                        "largest_over_smallest": growth, "flat": growth <= 1 + flat}
    result["flat"] = all(kind["flat"] for kind in result.values())
    return result


def table(report):
    sides = report["sides"]
    shape = (f"main reaching the modules through a tree of {report['fanout']} calls a module" if report["fanout"]
             else "bigpkg's own main, calling every module")
    head = (f"The compiler's peak memory against project size (ODR-281): bigpkg, {shape}; "
            f"{report['clean runs']} clean build(s) and {report['edits']} edit builds per size and compiler"
            + (f", the compilers interleaved" if len(sides) > 1 else "") + ". "
            + "; ".join(f"{side}: `{exe}`" for side, exe in sides.items()) + ".")
    columns = ["Lines", "Modules"]
    for side in sides:
        columns += [f"{side}: clean peak", f"{side}: clean time", f"{side}: after edits, highest peak", f"{side}: edit time"]
    if len(sides) == 2:
        columns += ["clean peak, after / before", "clean time, after / before", "edit time, after / before"]
    rows = ["| " + " | ".join(columns) + " |", "|" + "---|" * len(columns)]
    for row in report["sizes"]:
        cells = [f"{row['lines']:,}", f"{row['modules']:,}"]
        for side in sides:
            clean, edits = row[side]["clean"], row[side]["edits"]
            cells += [f"{clean['peak_mb']:.1f} MB", f"{clean['seconds']:.1f} s",
                      f"{edits['peak_mb']:.1f} MB" if edits["peak_mb"] is not None else "—",
                      f"{edits['seconds']:.2f} s" if edits["seconds"] is not None else "—"]
        if len(sides) == 2:
            before, after = (row[side] for side in sides)
            ratio = lambda a, b: f"{a / b:.3f}x" if a is not None and b else "—"
            cells += [ratio(after["clean"]["peak_mb"], before["clean"]["peak_mb"]), ratio(after["clean"]["seconds"], before["clean"]["seconds"]),
                      ratio(after["edits"]["seconds"], before["edits"]["seconds"])]
        rows.append("| " + " | ".join(cells) + " |")
    lines = [head, "\n".join(rows)]
    for side in sides:
        verdict = report["verdicts"][side]
        for kind in ("clean", "edits"):
            if kind in verdict:
                v = verdict[kind]
                lines.append(f"{side}, {'clean builds' if kind == 'clean' else 'after edits'}: {v['slope_mb_per_1000_lines']:.3f} MB per 1,000 lines "
                             f"(the largest size's peak {v['largest_over_smallest']:.2f}x the smallest's): {'flat' if v['flat'] else 'grows'}.")
    return "\n\n".join(lines) + "\n"


def run(ember, before, sizes, fanout, edits, runs, jobs, work, keep, flat, log=print):
    """The curve's report (a dict), measured; `before` may be `None`."""
    sides = {"before": os.path.abspath(before)} if before else {}
    sides["after"] = os.path.abspath(ember)
    os.makedirs(work, exist_ok=True)
    rows = measure(sides, sizes, fanout, edits, runs, jobs, os.path.abspath(work), keep, log)
    return {"schema": 1, "sides": sides, "fanout": fanout, "edits": edits, "clean runs": runs, "sizes": rows,
            "verdicts": {side: judge(rows, side, flat) for side in sides}, "flat": judge(rows, "after", flat)["flat"]}


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--ember", "--after", dest="ember", default=shutil.which(CLI) or os.path.join(ROOT, "target", "release", CLI + EXE))
    parser.add_argument("--before")
    parser.add_argument("--sizes", default="50000,100000,200000,400000,800000")
    parser.add_argument("--fanout", type=int, default=64)
    parser.add_argument("--edits", type=int, default=9)
    parser.add_argument("--runs", type=int, default=1, help="clean builds per size and compiler (the median is published)")
    parser.add_argument("--jobs", type=int, help="the compiler's -j (default its own: the physical cores)")
    parser.add_argument("--work", default=os.path.join(ROOT, "build", "memory-curve"))
    parser.add_argument("--keep", action="store_true", help="keep each size's packages and outputs")
    parser.add_argument("--flat", type=float, default=0.10)
    parser.add_argument("--json")
    parser.add_argument("--markdown")
    args = parser.parse_args(argv)
    sizes = [int(size) for size in args.sizes.split(",") if size.strip()]
    try:
        report = run(args.ember, args.before, sizes, args.fanout, args.edits, args.runs, args.jobs, args.work, args.keep, args.flat)
    except BuildFailed as failure:
        print(failure, file=sys.stderr)
        return 2
    if args.json:
        with open(args.json, "w", encoding="utf-8", newline="\n") as f:
            f.write(json.dumps(report, indent=2) + "\n")
    text = table(report)
    if args.markdown:
        with open(args.markdown, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
    print(text)
    return 0 if report["flat"] else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
