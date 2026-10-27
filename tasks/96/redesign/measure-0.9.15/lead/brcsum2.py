"""Medians of the counting designs' speed matrix (labels design_pattern_threads), pooled over the run
files given, each design against today's (atomic) of the same pattern and thread count."""
import re
import statistics
import sys

rows = {}
for f in sys.argv[1:]:
    for l in open(f, encoding="utf-8"):
        m = re.search(r"label=\[(\w+)\] exit=(\d+) out=\[\S+ \w+ threads=\d+ ms=([\d.]+)", l)
        if not m:
            continue
        design, pattern, threads = m[1].rsplit("_", 2)
        if m[2] != "0":
            print("EXIT", l.strip())
        rows.setdefault((pattern, int(threads)), {}).setdefault(design, []).append(float(m[3]))

designs = [d for d in ["atomic", "biased", "biased_chunk", "biased_pub", "biased_pubfast"] if any(d in v for v in rows.values())]
print("| pattern | threads | " + " | ".join(f"{d} ms (spread)" for d in designs) + " | "
      + " | ".join(f"{d} / atomic" for d in designs[1:]) + " |")
print("|---|---|" + "---|" * (2 * len(designs) - 1))
for (p, th), dd in rows.items():
    med = {d: statistics.median(dd[d]) for d in designs if d in dd}
    cells = [f"{med[d]:.1f} ({(max(dd[d]) - min(dd[d])) / min(dd[d]):.0%})" for d in designs if d in dd]
    ratios = [f"{med[d] / med['atomic']:.3f}" for d in designs[1:] if d in dd]
    print(f"| {p} | {th} | " + " | ".join(cells) + " | " + " | ".join(ratios) + " |")
