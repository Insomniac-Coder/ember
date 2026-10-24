# medians.py: summarise a run.ps1 output file: median, min, max ms and peak KB per program, ratio of the
# median to the first program named (the baseline). Usage: python -I medians.py <runs file> <baseline exe name>
import re, sys, statistics
path, base = sys.argv[1], sys.argv[2]
rows = {}
for line in open(path, encoding="utf-8-sig"):
    m = re.search(r"exe=(\S+) exit=(\S+) out=\[(.*)\]", line)
    if not m:
        continue
    exe, out = m.group(1), m.group(3)
    t = re.search(r"(\S+) ms=([\d.]+) peak_kb=(\d+) checksum=(\d+)", out)
    if not t:
        rows.setdefault(exe, {"ms": [], "kb": [], "ck": set(), "name": "?", "bad": True})
        continue
    r = rows.setdefault(exe, {"ms": [], "kb": [], "ck": set(), "name": t.group(1), "bad": False})
    r["ms"].append(float(t.group(2))); r["kb"].append(int(t.group(3))); r["ck"].add(t.group(4))
bm = statistics.median(rows[base]["ms"])
print(f"{'program':44} runs  median ms     min     max  ratio  peak KB  checksums")
for exe, r in rows.items():
    if not r["ms"]:
        print(f"{exe:44} no timed line"); continue
    md = statistics.median(r["ms"])
    print(f"{r['name']:44} {len(r['ms']):4} {md:10.3f} {min(r['ms']):7.1f} {max(r['ms']):7.1f} {md / bm:6.3f} {max(r['kb']):8d}  {sorted(r['ck'])}")
