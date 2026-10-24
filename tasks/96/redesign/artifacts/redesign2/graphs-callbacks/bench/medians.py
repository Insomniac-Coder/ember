# medians.py: median ms and peak KB per program from a run.ps1 output file, with ratios against the
# program named second. Usage: python -I medians.py <results file> <baseline name> [<second baseline>]
import re, sys, statistics
path, bases = sys.argv[1], sys.argv[2:]
rows = {}
for line in open(path):
    m = re.search(r"out=\[(\S+) ms=([\d.]+) peak_kb=(\d+) checksum=(\d+)", line)
    if m:
        rows.setdefault(m.group(1), []).append((float(m.group(2)), int(m.group(3)), m.group(4)))
def med(name, k): return statistics.median(r[k] for r in rows[name])
hdr = f"{'program':28} {'runs':>4} {'median ms':>10} {'min':>9} {'max':>9} {'spread':>7} {'peak KB':>8}"
for b in bases: hdr += f" {'t/' + b[:12]:>16} {'m/' + b[:12]:>16}"
print(hdr + "  checksums")
for name, rs in rows.items():
    ms = med(name, 0); kb = med(name, 1)
    lo = min(r[0] for r in rs); hi = max(r[0] for r in rs)
    line = f"{name:28} {len(rs):4} {ms:10.3f} {lo:9.3f} {hi:9.3f} {(hi - lo) / lo * 100:6.1f}% {kb:8.0f}"
    for b in bases: line += f" {ms / med(b, 0):16.3f} {kb / med(b, 1):16.3f}"
    print(line + "  " + ",".join(sorted(set(r[2] for r in rs))))
