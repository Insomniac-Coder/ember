"""`[BLD-10]` (ODR-242), `[HR-1]`, `[HR-9]` — the compiler's speed, measured on one machine before
and after a change, and published.

ODR-242 (the owner, 2026-10-22): no fixed time limits, since a time varies from CPU to CPU; the
compiler is made as fast as it can be. So a change's effect on the compiler's work is measured on
one machine: the compiler before the change (`--before`) and after it (`--after`) do the same work
over the same generated package (`bench/bigpkg`, `[BUD-1]`), their runs interleaved — each round
runs both, the first of them in turn — so what the machine does meanwhile falls on both alike.
A change may not make the compiler slower: a figure is *slower* when the after compiler took
longer in so many of the rounds that chance alone would do so less than once in a hundred (a
one-sided sign test over the rounds), and the gate fails. Every figure is published with both
compilers' medians and spreads (`[BUD-3b]`: the interquartile range over the median), their ratio
and the verdict, as a JSON report (`--json`, which holds every run) and a Markdown table
(`--markdown`); `--judge REPORT` gives a published report's verdicts again from its runs.

The figures, each the median of `--runs` rounds after `--warmups` (15 and 3, `[BUD-2]`):

    B1-B8  [BUD-2]'s operations over bigpkg (B6 with a new, empty global cache; B7 without the
           program cache);
    B9     the compiler's peak memory in a clean build; [BUD-2]'s 3 GB stands, being no time;
    HR-1   an edit to one function body of the reloadable bigpkg, running in its process: from
           the save, through `ember build --reload` (B2) and the runtime's load, plan, prepare
           and commit, to the program's first result from the new body, with [BEN-8]'s 10,000
           live instances all still there (`bench/bigpkg/generate.py --live`);
    HR-9   what reload costs a program that is not reloading, the after compiler's:
           `ember bench --reload` over the performance suite; [HR-9]'s 3 % stands, being a
           ratio, no time (`--hr9-dir`, `--hr9-only`, `--hr9-runs`, `--hr9-core`).

Without `--before` the after compiler's figures are measured and published, and only B9's and
HR-9's bounds are gates. The release workflow publishes the figures of each release against the
one before it; the gate of record is the run on the reference laptop before a push.

    python tools/compile_budgets.py [--after EXE] [--before EXE] [--work DIR] [--runs N]
        [--warmups N] [--modules N] [--lines N] [--header-lines N] [--only B1,HR-1]
        [--json FILE] [--markdown FILE] [--judge REPORT] [--list]

Exit status: 0 no figure is slower and every bound holds; 1 otherwise.
"""
import argparse
import importlib.util
import json
import math
import os
import queue
import re
import shutil
import statistics
import subprocess
import sys
import threading
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIGPKG = os.path.join(ROOT, "bench", "bigpkg")
# The compiler's names (`ember_branding`): its executable, its variables.
CLI = "ember"
CACHE_VAR = "EMBER_CACHE"
NO_COMPILE_CACHE_VAR = "EMBER_NO_COMPILE_CACHE"
HOT_VAR = "EMBER_HOT"
EXE = ".exe" if os.name == "nt" else ""

FIGURES = {
    "B1": "`ember check` after a one-function-body edit",
    "B2": "`ember build --reload` after a one-function-body edit",
    "B3": "`ember build` after a one-function-body edit, `debug`",
    "B4": "`ember build` after a signature change in a leaf module",
    "B5": "`ember build` after a change to a widely imported type",
    "B6": "clean `debug` build, cold cache",
    "B7": "clean `development` build",
    "B8": "`.embind` regeneration for a Vulkan-sized header",
    "B9": "compiler peak memory, clean build",
    "HR-1": "an edit to one function body running in the live process, 10,000 instances kept",
    "HR-9": "the performance suite's overhead with reload on (`ember bench --reload`)",
}
# The bounds that are no time limits: B9's 3 GB ([BUD-2]) and HR-9's 3 % ([HR-9]).
BOUNDS = {"B9": 3 * 1024, "HR-9": 3.0}
UNITS = {"B9": "MB", "HR-9": "%"}
# Measured once, by the after compiler: a bound, not a comparison.
ONCE = {"HR-9"}
SIGNIFICANCE = 0.01


def load_generator():
    spec = importlib.util.spec_from_file_location("bigpkg_generate", os.path.join(BIGPKG, "generate.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class TimedOut(Exception):
    """A command that ran past its limit, ended with everything it started."""


def end_tree(process):
    if process.poll() is not None:
        return
    if os.name == "nt":
        subprocess.run(["taskkill", "/T", "/F", "/PID", str(process.pid)], capture_output=True)
    else:
        import signal
        os.killpg(process.pid, signal.SIGKILL)
    process.wait()


class Runner:
    """One compiler's side: its executable, its own work folder and global cache."""

    def __init__(self, ember, work):
        self.ember = ember
        self.work = work
        self.env = {CACHE_VAR: os.path.join(work, "cache")}
        # Each run's limit, in seconds.
        self.limit = 600.0

    def run(self, args, cwd, env=None):
        """Run the compiler; its wall time in milliseconds, keeping what it wrote on standard error
        (`self.stderr`). A failing command stops the measurement: a figure of a failed build is no
        figure. One that runs past `self.limit` is ended and raises `TimedOut`."""
        environment = dict(os.environ)
        environment.update(self.env)
        environment.update(env or {})
        start = time.perf_counter()
        process = subprocess.Popen([self.ember, *args], cwd=cwd, env=environment, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, start_new_session=os.name != "nt")
        try:
            out, err = process.communicate(timeout=self.limit)
        except subprocess.TimeoutExpired:
            end_tree(process)
            raise TimedOut(f"`{CLI} {' '.join(args)}` ran past {self.limit:.0f} s")
        elapsed = (time.perf_counter() - start) * 1000
        self.stderr = err.decode(errors="replace")
        if process.returncode != 0:
            raise RuntimeError(f"`{CLI} {' '.join(args)}` failed in {cwd}:\n{out.decode(errors='replace')}{self.stderr}")
        return elapsed

    def peak_memory_mb(self):
        """B9 — the compiler's own peak memory, from its `--timings=json` report."""
        for line in self.stderr.splitlines():
            if line.startswith('{"'):
                peak = json.loads(line).get("peak_memory_mb")
                if peak is not None:
                    return peak
        raise RuntimeError("the compiler reported no peak memory (`--timings=json`)")


def fresh(path):
    shutil.rmtree(path, ignore_errors=True)
    os.makedirs(path, exist_ok=True)
    return path


class Live:
    """`[HR-1]` — the live reloadable bigpkg: built, then its program started and kept running; each
    measurement saves an edit to the middle module's `work` body, builds the next image (`ember
    build --reload`) and waits until the program prints the new body's first result with every
    instance still there."""

    def __init__(self, runner, generator, size):
        self.runner, self.generator, self.size = runner, generator, size
        self.pkg = os.path.join(runner.work, "pkg-live")
        self.process = None

    def setup(self):
        fresh(self.pkg)
        self.generator.generate(self.pkg, *self.size, live=True)
        self.runner.run(["build"], self.pkg)
        program = os.path.join(self.pkg, "target", "debug", "bin", "main" + EXE)
        env = dict(os.environ, **{HOT_VAR: "1"})  # each line written as the program writes it
        self.process = subprocess.Popen([program], cwd=os.path.dirname(program), env=env, stdout=subprocess.PIPE,
                                        stderr=subprocess.DEVNULL, start_new_session=os.name != "nt")
        self.lines = queue.Queue()
        threading.Thread(target=self.read, args=(self.process.stdout,), daemon=True).start()
        self.value = self.wait(None)

    def read(self, stream):
        for line in stream:
            self.lines.put(line.decode(errors="replace").strip())
        self.lines.put(None)

    def wait(self, previous):
        """The program's next result other than `previous`, every instance still there."""
        deadline = time.perf_counter() + self.runner.limit
        while True:
            try:
                line = self.lines.get(timeout=max(deadline - time.perf_counter(), 0.001))
            except queue.Empty:
                raise TimedOut(f"the live program printed no new result in {self.runner.limit:.0f} s")
            if line is None:
                raise RuntimeError("the live program ended")
            words = line.split()
            if len(words) == 4 and words[0] == "work" and words[1] != previous:
                if int(words[3]) != self.generator.LIVE_INSTANCES:
                    raise RuntimeError(f"the live program kept {words[3]} of its {self.generator.LIVE_INSTANCES} instances [HR-1]")
                return words[1]

    def measure(self, n):
        start = time.perf_counter()
        self.generator.edit(self.pkg, "body", n, self.size[0], self.size[1])
        self.runner.run(["build", "--reload"], self.pkg)
        self.value = self.wait(self.value)
        return (time.perf_counter() - start) * 1000

    def teardown(self):
        if self.process is not None:
            end_tree(self.process)
            self.process = None


def operations(runner, generator, size):
    """Each figure's setup, one measured run (given the run's number, so every edit is new) and
    teardown, for one compiler's side."""
    work = runner.work
    pkg = os.path.join(work, "pkg")
    reload_pkg = os.path.join(work, "pkg-reload")
    edits = {"body": 0, "signature": 0, "type": 0}
    nothing = lambda: None

    def edited(kind):
        edits[kind] += 1
        return edits[kind]

    def setup_pkg():
        generator.generate(pkg, *size)
        runner.run(["build"], pkg)

    def setup_reload():
        generator.generate(reload_pkg, *size)
        with open(os.path.join(reload_pkg, "ember.toml"), "a", encoding="utf-8") as manifest:
            manifest.write('\n[build]\nreload = "all"\n')
        runner.run(["build"], reload_pkg)

    def edit_then(kind, args, directory=None):
        def measure(_):
            generator.edit(directory or pkg, kind, edited(kind), size[0], size[1])
            return runner.run(args, directory or pkg)
        return measure

    def clean(args, cold):
        def measure(n):
            target = fresh(os.path.join(work, "clean-target"))
            env = {NO_COMPILE_CACHE_VAR: "1"}
            if cold:
                env[CACHE_VAR] = fresh(os.path.join(work, f"cold-cache-{n}"))
            return runner.run([*args, "--out-dir", target], pkg, env)
        return measure

    def bind(_):
        return runner.run(["bind", os.path.join("src", "vkimpl.h"), "--emit-embind"], pkg)

    clean_build = clean(["build", "--timings=json"], cold=False)

    def peak(n):
        clean_build(n)
        return runner.peak_memory_mb()

    live = Live(runner, generator, size)
    return {
        "B1": (setup_pkg, edit_then("body", ["check"]), nothing),
        "B2": (setup_reload, edit_then("body", ["build", "--reload"], reload_pkg), nothing),
        "B3": (setup_pkg, edit_then("body", ["build"]), nothing),
        "B4": (setup_pkg, edit_then("signature", ["build"]), nothing),
        "B5": (setup_pkg, edit_then("type", ["build"]), nothing),
        "B6": (setup_pkg, clean(["build"], cold=True), nothing),
        "B7": (setup_pkg, clean(["build", "--profile", "development"], cold=False), nothing),
        "B8": (setup_pkg, bind, nothing),
        "B9": (setup_pkg, peak, nothing),
        "HR-1": (live.setup, live.measure, live.teardown),
    }


def hr9(ember, args):
    """`[HR-9]` — `ember bench --reload`'s suite overhead, in percent, and its configuration line."""
    command = [ember, "bench", "--reload", "--dir", args.hr9_dir, "--runs", str(args.hr9_runs), "--core", str(args.hr9_core), "--report"]
    if args.hr9_only:
        command.insert(2, args.hr9_only)
    done = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, errors="replace", timeout=6 * 3600)
    found = re.search(r"suite overhead \(geometric mean of \d+ programs\): ([+-]?\d+(?:\.\d+)?) %", done.stdout)
    if done.returncode != 0 or not found:
        raise RuntimeError(f"`{CLI} bench --reload` measured nothing:\n{done.stdout[-3000:]}{done.stderr[-3000:]}")
    configuration = next((line for line in done.stdout.splitlines() if line.startswith("[HR-9]")), "")
    return float(found.group(1)), configuration


def spread(samples):
    """`[BUD-3b]` — the interquartile range over the median."""
    median = statistics.median(samples)
    if not median:
        return 0.0
    if len(samples) < 4:
        return (max(samples) - min(samples)) / median
    q1, _, q3 = statistics.quantiles(samples, n=4)
    return (q3 - q1) / median


def tail(n, k):
    """P(at least `k` of `n` fair coin tosses): a one-sided sign test's p-value."""
    return sum(math.comb(n, i) for i in range(k, n + 1)) / 2 ** n if n else 1.0


def judge(name, before, after):
    """A figure's verdict from its runs: `slower` (fails), `faster`, `no measurable change`; a bound's
    `over its bound` (fails) or `within its bound`; `published` with no compiler to compare."""
    result = {"operation": FIGURES[name], "unit": UNITS.get(name, "ms"),
              "after": {"median": statistics.median(after), "spread": spread(after), "runs": after}}
    reasons = []
    if before:
        rounds = list(zip(before, after))
        slower = sum(a > b for b, a in rounds)
        faster = sum(a < b for b, a in rounds)
        ratio = statistics.median(after) / statistics.median(before) if statistics.median(before) else 1.0
        p_slower, p_faster = tail(slower + faster, slower), tail(slower + faster, faster)
        result["before"] = {"median": statistics.median(before), "spread": spread(before), "runs": before}
        result.update({"ratio": ratio, "rounds slower": slower, "rounds faster": faster, "p": min(p_slower, p_faster)})
        if p_slower < SIGNIFICANCE and ratio > 1:
            result["change"] = "slower" if name != "B9" else "more memory"
            # Memory is no speed: B9's gate is its bound (ODR-242 is about time).
            if name != "B9":
                reasons.append(f"slower in {slower} of {len(rounds)} rounds (p {p_slower:.4f} < {SIGNIFICANCE}), {ratio:.3f}x")
        elif p_faster < SIGNIFICANCE and ratio < 1:
            result["change"] = "faster" if name != "B9" else "less memory"
        else:
            result["change"] = "no measurable change"
    bound = BOUNDS.get(name)
    if bound is not None:
        result["bound"] = bound
        if result["after"]["median"] > bound:
            reasons.append(f"over its bound ({result['after']['median']:.1f} > {bound} {result['unit']})")
    result["verdict"] = "fail" if reasons else "pass"
    if reasons:
        result["why"] = "; ".join(reasons)
    return result


def table(report):
    """The figures as a Markdown table."""
    rows = ["| Figure | Operation | Before | After | Change | Verdict |", "|---|---|---|---|---|---|"]
    for name, result in report["figures"].items():
        unit = result["unit"]
        shown = lambda side: f"{side['median']:.1f} {unit} (spread {side['spread']:.1%})" if side else "—"
        change = f"{result['change']}, {result['ratio']:.3f}x" if "ratio" in result else ("measured once" if name in ONCE else "—")
        verdict = result["verdict"] + (f": {result['why']}" if "why" in result else "")
        rows.append(f"| {name} | {result['operation']} | {shown(result.get('before'))} | {shown(result['after'])} | {change} | {verdict} |")
    head = [f"Compiler speed, {report['machine']['processor'] or report['machine']['machine']} ({report['machine']['system']}, "
            f"{report['machine']['cores']} logical processors); bigpkg {report['size']['modules']} modules, "
            f"{report['size']['lines']} lines; {report['rounds']} rounds after {report['warmups']} warm-up rounds."]
    if report.get("hr9"):
        head.append(f"HR-9: {report['hr9']}")
    return "\n\n".join(head + ["\n".join(rows)]) + "\n"


def show(report):
    for name, result in report["figures"].items():
        unit = result["unit"]
        before = f"{result['before']['median']:>9.1f}{unit:>3} ({result['before']['spread']:>5.1%})" if "before" in result else " " * 20
        after = f"{result['after']['median']:>9.1f}{unit:>3} ({result['after']['spread']:>5.1%})"
        change = f"{result['change']} {result['ratio']:.3f}x" if "ratio" in result else ""
        why = f": {result['why']}" if "why" in result else ""
        print(f"{name:5} {before} {after}  {change:32} {result['verdict']}{why}  ({result['operation']})")
    failed = [name for name, result in report["figures"].items() if result["verdict"] == "fail"]
    if failed:
        print(f"the compiler's speed: FAILED ({', '.join(failed)}) [BLD-10]")
        return 1
    print("the compiler's speed: passed [BLD-10]")
    return 0


def machine():
    import platform
    return {"system": platform.system(), "machine": platform.machine(), "processor": platform.processor(),
            "cores": os.cpu_count()}


def write(path, text):
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--after", "--ember", dest="after", default=shutil.which(CLI) or os.path.join(ROOT, "target", "release", CLI + EXE))
    parser.add_argument("--before")
    parser.add_argument("--work", default=os.path.join(ROOT, "build", "budgets"))
    parser.add_argument("--runs", type=int, default=15)
    parser.add_argument("--warmups", type=int, default=3)
    parser.add_argument("--modules", type=int, default=400)
    parser.add_argument("--lines", type=int, default=50000)
    parser.add_argument("--header-lines", type=int, default=20000)
    parser.add_argument("--only", default=",".join(FIGURES))
    parser.add_argument("--hr9-dir", default=os.path.join(ROOT, "tests", "perf"))
    parser.add_argument("--hr9-only")
    parser.add_argument("--hr9-runs", type=int, default=30)
    parser.add_argument("--hr9-core", type=int, default=1)
    parser.add_argument("--json")
    parser.add_argument("--markdown")
    parser.add_argument("--judge", help="a published report: its verdicts again, from its runs")
    parser.add_argument("--list", action="store_true", help="print each figure's operation and bound as JSON")
    args = parser.parse_args(argv)
    if args.list:
        print(json.dumps({name: {"operation": what, **({"bound": BOUNDS[name], "unit": UNITS[name]} if name in BOUNDS else {})}
                          for name, what in FIGURES.items()}))
        return 0
    if args.judge:
        with open(args.judge, encoding="utf-8") as f:
            report = json.load(f)
        report["figures"] = {name: judge(name, result.get("before", {}).get("runs"), result["after"]["runs"])
                             for name, result in report["figures"].items()}
        return show(report)

    wanted = [name.strip() for name in args.only.split(",") if name.strip()]
    unknown = [name for name in wanted if name not in FIGURES]
    if unknown:
        parser.error(f"unknown figure(s): {', '.join(unknown)}")
    generator = load_generator()
    work = os.path.abspath(fresh(args.work))
    sides = {"after": Runner(os.path.abspath(args.after), fresh(os.path.join(work, "after")))}
    if args.before:
        sides["before"] = Runner(os.path.abspath(args.before), fresh(os.path.join(work, "before")))
    size = (args.modules, args.lines, args.header_lines)
    ops = {side: operations(runner, generator, size) for side, runner in sides.items()}
    figures = {}
    hr9_configuration = None
    for name in wanted:
        if name == "HR-9":
            overhead, hr9_configuration = hr9(sides["after"].ember, args)
            figures[name] = judge(name, None, [overhead])
            print(f"{name}: measured", flush=True)
            continue
        order = list(sides)
        runs = {side: [] for side in sides}
        try:
            for side in order:
                ops[side][name][0]()
            for n in range(args.warmups + args.runs):
                # Each round runs both compilers, the first of them in turn.
                for side in (order if n % 2 == 0 else order[::-1]):
                    value = ops[side][name][1](n)
                    if n >= args.warmups:
                        runs[side].append(value)
        finally:
            for side in order:
                ops[side][name][2]()
        figures[name] = judge(name, runs.get("before"), runs["after"])
        print(f"{name}: measured", flush=True)
    report = {"schema": 2, "machine": machine(), "size": {"modules": size[0], "lines": size[1], "header_lines": size[2]},
              "rounds": args.runs, "warmups": args.warmups, "after": sides["after"].ember,
              "before": sides["before"].ember if "before" in sides else None, "hr9": hr9_configuration, "figures": figures}
    if args.json:
        write(args.json, json.dumps(report, indent=2) + "\n")
    if args.markdown:
        write(args.markdown, table(report))
    return show(report)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
