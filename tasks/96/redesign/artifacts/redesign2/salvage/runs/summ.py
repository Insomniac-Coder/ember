"""summ.py <results.txt> <baseline exe stem>: median ms / peak KB per program, ratio to the baseline,
spread (max-min)/min, and whether every run printed the same checksum as the baseline."""
import re, sys, statistics as st

rows = {}
for line in open(sys.argv[1]):
    m = re.match(r"round=(\d+) exe=(\S+)\.exe out=\[(.*)\]", line)
    if not m:
        continue
    r = re.search(r"ms=([\d.]+) peak_kb=(\d+) checksum=(\d+)", m.group(3))
    if not r:
        rows.setdefault(m.group(2), []).append((None, None, "FAIL " + m.group(3)))
        continue
    rows.setdefault(m.group(2), []).append((float(r.group(1)), int(r.group(2)), r.group(3)))
base = sys.argv[2]
bms = st.median(x[0] for x in rows[base])
bkb = st.median(x[1] for x in rows[base])
bsum = rows[base][0][2]
print(f"{'program':28} {'n':>2} {'med ms':>9} {'x base':>7} {'med KB':>8} {'x base':>7} {'spread':>7} checksum")
for k, v in rows.items():
    ms = [x[0] for x in v if x[0] is not None]
    kb = [x[1] for x in v if x[1] is not None]
    ok = all(x[2] == bsum for x in v)
    med, mkb = st.median(ms), st.median(kb)
    print(f"{k:28} {len(ms):>2} {med:9.3f} {med / bms:7.3f} {mkb:8d} {mkb / bkb:7.3f} {(max(ms) - min(ms)) / min(ms):7.1%} {'same' if ok else 'DIFFERENT'}")
