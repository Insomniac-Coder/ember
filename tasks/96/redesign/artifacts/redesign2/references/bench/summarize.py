# summarize.py: medians of results.txt (work_ms if printed, else ms; peak_kb), spread, checksum agreement.
import re, statistics, sys
rows = {}
for line in open(sys.argv[1]):
    m = re.match(r"round=(\d+) prog=(\S+) out=\[(.*)\] err=\[(.*)\]", line.strip())
    if not m: continue
    out = m.group(3)
    d = dict(re.findall(r"(\w+)=([\d.]+)", out))
    rows.setdefault(m.group(2), []).append(d)
for prog, ds in rows.items():
    ms = [float(d.get('work_ms', d['ms'])) for d in ds]
    tot = [float(d['ms']) for d in ds]
    kb = [int(d['peak_kb']) for d in ds]
    cs = {d['checksum'] for d in ds}
    print(f"{prog:22s} n={len(ds)} work_med={statistics.median(ms):9.3f} spread={100*(max(ms)-min(ms))/min(ms):5.1f}% "
          f"total_med={statistics.median(tot):9.3f} peak_kb_med={int(statistics.median(kb)):7d} checksums={len(cs)} {sorted(cs)[0]}")
