# medians.py: median ms and peak KB per program from a run.ps1 output file; ratios against the first name given.
import re, sys, statistics
path, base = sys.argv[1], sys.argv[2]
rows = {}
for line in open(path):
    m = re.search(r"out=\[(\S+) ms=([\d.]+) peak_kb=(\d+) checksum=(\d+)\]", line)
    if m:
        rows.setdefault(m.group(1), []).append((float(m.group(2)), int(m.group(3)), m.group(4)))
b_ms = statistics.median(r[0] for r in rows[base]); b_kb = statistics.median(r[1] for r in rows[base])
print(f"{'program':32} {'runs':>4} {'median ms':>10} {'min':>9} {'max':>9} {'ratio':>6} {'peak KB':>8} {'mem':>5} checksums")
for name, rs in rows.items():
    ms = statistics.median(r[0] for r in rs); kb = statistics.median(r[1] for r in rs)
    print(f"{name:32} {len(rs):4} {ms:10.3f} {min(r[0] for r in rs):9.3f} {max(r[0] for r in rs):9.3f} {ms/b_ms:6.2f} {kb:8.0f} {kb/b_kb:5.2f} {sorted(set(r[2] for r in rs))}")
