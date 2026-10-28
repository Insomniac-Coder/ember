"""Medians of the measuring phase's runs: python summarize.py <wl> [<suffix>] -> a markdown table.
Ratio = median ms / the median of the workload's baseline from the same compiler: the first `plain_` program
(its `_msvc` twin for MSVC programs; clang's for clang and Rust). respan's `_rtlen` programs use plain_respan_rtlen.
Spread = (slowest - fastest) / fastest. ok = every run exited 0 and printed the workload's checksum."""
import re
import statistics
import sys
from pathlib import Path

HERE = Path(__file__).parent
wl = sys.argv[1]
suffixes = sys.argv[2].split(",") if len(sys.argv) > 2 else [""]  # several sets pooled: "",b,c
rows = [l.split("\t") for l in (HERE / "lists" / f"{wl}.tsv").read_text(encoding="utf-8").splitlines() if l.strip()]
expected = {r[0]: (r[3] if len(r) > 3 else "") for r in rows}
order = [r[0] for r in rows]
text = []
for suffix in suffixes:
    t = (HERE / "runs" / f"{wl}{suffix}.txt").read_text(encoding="utf-8").splitlines()
    assert t[-1] == "DONE", "the run did not finish"
    text += t
pat = re.compile(r"label=\[(.+?)\] exit=(-?\d+) out=\[\S+ ms=([\d.]+) peak_kb=(\d+) checksum=(\d+)\]")
data, bad = {}, []
for l in text:
    m = pat.search(l)
    if m:
        data.setdefault(m[1], []).append((float(m[3]), int(m[4]), m[5], int(m[2])))
    elif l.startswith("round="):
        bad.append(l)
first_plain = next(o for o in order if o.startswith("plain_") and "_msvc" not in o)
agree = expected[first_plain] or (data[first_plain][0][2] if first_plain in data else "")


def base_of(label):
    msvc = "_msvc" in label
    rt = "_rtlen" in label
    for o in order:
        if o.startswith("plain_") and ("_msvc" in o) == msvc and ("_rtlen" in o) == rt and o.split(" ")[1:] == label.split(" ")[1:]:
            return o
    for o in order:
        if o.startswith("plain_") and ("_msvc" in o) == msvc and ("_rtlen" in o) == rt:
            return o
    return first_plain


med = {k: statistics.median(x[0] for x in v) for k, v in data.items()}
print(f"| {wl} program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |")
print("|---|---|---|---|---|---|---|")
for k in order:
    if k not in data:
        print(f"| {k} | 0 | | | | | missing |")
        continue
    v = data[k]
    ms = [x[0] for x in v]
    want = expected[k] or agree
    ok = all(x[3] == 0 and (not want or x[2] == want) for x in v)
    b = base_of(k)
    ratio = round(med[k] / med[b], 3) if b in med else ""
    print(f"| {k} | {len(v)} | {statistics.median(ms):.1f} | {round(statistics.median(x[1] for x in v))} | {ratio} | {(max(ms) - min(ms)) / min(ms):.1%} | {'yes' if ok else 'NO ' + str(sorted({x[2] for x in v}))} |")
if bad:
    print("\nlines that did not parse:")
    for l in bad:
        print("    " + l)
