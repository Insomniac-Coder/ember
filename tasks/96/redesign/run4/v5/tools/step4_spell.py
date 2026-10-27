"""Step 4, phase 0: the spelling changes that need no judgement (verdicts 83 and 86).

In program code only (test programs; ```ember blocks under NEW and Still refused in the example
files), never in OLD code, today's outputs, "means exactly" programs, comments, `#$` lines or prose:
  - `with` before what a function keeps goes: `fn() with copy of score` -> `fn() copy of score` (83);
  - `changes ...` and `reads ...` clauses go from `fn` lines and function types (86).
Writes a report of every changed line, every line it left with a removed word in it, and the tests
whose expected error belonged to the removed words. Run with no arguments on run4/v5; `--check` runs
the self-checks only.
"""
import re
import sys
from pathlib import Path

V5 = Path(__file__).resolve().parents[1]
CAPTURE = r"(?:copy of|owns|shares|thread_shared|weakref)"
FN_PARAMS = r"\bfn(?:\s+[A-Za-z_]\w*)?\s*\((?:[^()]|\([^()]*\))*\)"
WITH_RE = re.compile(r"(" + FN_PARAMS + r")\s+with\s+(?=" + CAPTURE + r"\b)")
KEYWORD_RE = re.compile(r"(?<![\w.])(changes|reads)\s+(?![=.(\[])")
TYPE_START = re.compile(
    r"[A-Za-z_]\w*:\s+(?:int|float|bool|str|u8|u16|u32|u64|i8|i16|i32|i64|f32|f64|usize|"
    r"[A-Z]\w*|fn\b|ref\b|owns\b|shares\b|thread_shared\b|weakref\b|sharedref\b|const\b|\*)"
)
GONE_CODES = ("E2105", "E2107", "E2113", "E2122", "E2085", "E0118")


def split_comment(line):
    """(code, rest): rest starts at the first `#` outside a string literal."""
    quote = None
    for i, ch in enumerate(line):
        if quote:
            if ch == "\\":
                continue
            if ch == quote:
                quote = None
        elif ch in "\"'":
            quote = ch
        elif ch == "#":
            return line[:i], line[i:]
    return line, ""


def parse_item(s, i):
    """End of one item of a `changes`/`reads` list starting at s[i], or None."""
    m = re.match(r"captured\.\.\.|[A-Za-z_][\w.]*", s[i:])
    if not m:
        return None
    j = i + m.end()
    while j < len(s) and s[j] == "[":
        depth = 0
        while j < len(s):
            if s[j] == "[":
                depth += 1
            elif s[j] == "]":
                depth -= 1
                if depth == 0:
                    j += 1
                    break
            j += 1
        else:
            return None
    return j


def open_depth(s, end):
    depth = 0
    for ch in s[:end]:
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
    return depth


def strip_effects(code):
    """Remove every `changes`/`reads` clause from a line of code; (new_code, unsure)."""
    unsure = False
    pos = 0
    while True:
        m = KEYWORD_RE.search(code, pos)
        if not m:
            break
        if not re.search(r"\bfn\b", code[: m.start()]):
            pos = m.end()
            continue
        nested = open_depth(code, m.start()) > 0
        j = m.end()
        while True:
            end = parse_item(code, j)
            if end is None:
                unsure = True
                break
            j = end
            if code.startswith(", ", j):
                rest = code[j + 2:]
                if nested and TYPE_START.match(rest):
                    break
                if re.match(r"(?:changes|reads)\b", rest):
                    break
                j += 2
                continue
            break
        start = m.start()
        while start > 0 and code[start - 1] == " ":
            start -= 1
        code = code[:start] + code[j:]
        pos = start
    return code, unsure


def respell(line):
    """New spelling of one line of program code; (line, unsure)."""
    code, rest = split_comment(line)
    new = WITH_RE.sub(r"\1 ", code)
    new, unsure = strip_effects(new)
    if re.search(r"(?<![\w.])(changes|reads)\s", new) or re.search(r"\bwith\s+" + CAPTURE + r"\b", new):
        unsure = True
    return new + rest, unsure


def respell_test(path, report, flagged):
    lines = path.read_text(encoding="utf-8").split("\n")
    out = []
    for n, line in enumerate(lines, 1):
        if line.lstrip().startswith(("#$", "##", "#")):
            out.append(line)
            continue
        new, unsure = respell(line)
        if new != line:
            report.append((path, n, line, new))
        if unsure:
            flagged.append((path, n, new))
        out.append(new)
    path.write_text("\n".join(out), encoding="utf-8", newline="\n")


SECTION = re.compile(r"^\s*(?:\*\*)?\d+\.\s*(?:\*\*)?\s*([A-Za-z][A-Za-z' ]*)")


def respell_area(path, report, flagged):
    lines = path.read_text(encoding="utf-8").split("\n")
    out, section, in_block = [], "", False
    for n, line in enumerate(lines, 1):
        stripped = line.strip()
        if not in_block:
            m = SECTION.match(line)
            if m and ("NEW" in m.group(1) or m.group(1).strip().startswith(
                    ("OLD", "Means", "Still", "Intent", "The owner", "Cost", "Kind"))):
                section = m.group(1).strip()
            if stripped.startswith("```"):
                in_block = stripped.startswith("```ember") and (
                    section.startswith("NEW") or section.startswith("Still"))
                out.append(line)
                continue
            out.append(line)
            continue
        if stripped.startswith("```"):
            in_block = False
            out.append(line)
            continue
        new, unsure = respell(line)
        if new != line:
            report.append((path, n, line, new))
        if unsure:
            flagged.append((path, n, new))
        out.append(new)
    path.write_text("\n".join(out), encoding="utf-8", newline="\n")


def self_check():
    cases = [
        ("    b.on_click(fn() changes score: score += 1)", "    b.on_click(fn(): score += 1)"),
        ("fn note(msg: String) changes log:", "fn note(msg: String):"),
        ("    fn load_assets() reads renderer -> Array[String]:", "    fn load_assets() -> Array[String]:"),
        ("fn main() changes FRAME, LOG, RESULTS:", "fn main():"),
        ("    handlers: Array[fn() changes captured, LOG] = []", "    handlers: Array[fn()] = []"),
        ("    fn on_click(owns f: fn() changes LOG) changes handlers:", "    fn on_click(owns f: fn()):"),
        ("fn f(g: fn() changes X, n: int) -> int:", "fn f(g: fn(), n: int) -> int:"),
        ("    reset = fn() with owns players changes Player: players.clear()",
         "    reset = fn() owns players: players.clear()"),
        ("    return fn() with owns count -> int:", "    return fn() owns count -> int:"),
        ("        scope.spawn(fn() with copy of x changes stats: stats.total += x)",
         "        scope.spawn(fn() copy of x: stats.total += x)"),
        ("fn dump(log: thread_shared Log, note: fn() changes Log) changes Log:",
         "fn dump(log: thread_shared Log, note: fn()):"),
        ("fn clear(scores: Mutex[Array[String]]) changes Mutex[Array[String]]:",
         "fn clear(scores: Mutex[Array[String]]):"),
        ("    fn for_each(owns f: F) owns this changes C:", "    fn for_each(owns f: F) owns this:"),
        ("    fn() changes score, lives: score += 1", "    fn(): score += 1"),
        ("    with g = scores.lock():", "    with g = scores.lock():"),
        ("    changes = 3", "    changes = 3"),
        ("    fn regraft(n: Node) with weakref root:", "    fn regraft(n: Node) weakref root:"),
        ("    x = 1    # this changes x", "    x = 1    # this changes x"),
        ("    bus.on_event(fn() with thread_shared count changes Counter: count.n.fetch_add(1))",
         "    bus.on_event(fn() thread_shared count: count.n.fetch_add(1))"),
    ]
    for before, want in cases:
        got, _ = respell(before)
        assert got == want, (before, got, want)
    print("self-check: %d cases pass" % len(cases))


def main():
    self_check()
    if "--check" in sys.argv:
        return
    report, flagged = [], []
    for path in sorted((V5 / "conformance").rglob("*.em")):
        respell_test(path, report, flagged)
    for path in sorted(V5.glob("area-*.md")):
        respell_area(path, report, flagged)
    gone = []
    for path in sorted((V5 / "conformance").rglob("*.em")):
        text = path.read_text(encoding="utf-8")
        codes = [c for c in GONE_CODES if "error[%s]" % c in text or "warning[%s]" % c in text]
        if codes:
            gone.append((path, codes))
    rel = lambda p: p.relative_to(V5).as_posix()
    with open(V5 / "tools" / "step4_spell_report.md", "w", encoding="utf-8", newline="\n") as f:
        f.write("# Step 4 spelling script: report\n\n")
        f.write("%d lines changed, %d lines left with a removed word in their code (for the agents), "
                "%d tests whose expected error belonged to the removed words.\n\n"
                % (len(report), len(flagged), len(gone)))
        f.write("## 1. Tests whose expected error belonged to the removed words\n\n")
        for p, codes in gone:
            f.write("- `%s`: %s\n" % (rel(p), ", ".join(codes)))
        f.write("\n## 2. Lines left with a removed word in their code\n\n")
        for p, n, line in flagged:
            f.write("- `%s:%d`: `%s`\n" % (rel(p), n, line.strip()))
        f.write("\n## 3. Every changed line (before, then after)\n\n")
        for p, n, before, after in report:
            f.write("- `%s:%d`\n  - `%s`\n  - `%s`\n" % (rel(p), n, before.strip(), after.strip()))
    print("changed %d lines, flagged %d, tests with gone codes %d" % (len(report), len(flagged), len(gone)))


if __name__ == "__main__":
    main()
