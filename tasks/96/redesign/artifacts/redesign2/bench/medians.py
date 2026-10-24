# medians.py: medians, spreads, ratios and checksum checks for runs/w1..w4.txt.
# Usage: python -I medians.py [suffix]   (suffix "" = batch a, "b" = batch b, "c" = batch c (W2 only); prints the summary)
import re, statistics, pathlib, sys
SUF = sys.argv[1] if len(sys.argv) > 1 else ""
RUNS = pathlib.Path(__file__).parent / "runs"
KNOWN = {"w1": 2111831520, "w2": 647034814, "w3": 9288233970, "w4": 34946993000}
BASE = {"w1": ("plain_w1_group.exe", "rust_w1.exe"), "w2": ("plain_w2.exe", "rust_w2.exe"),
        "w3": ("plain_w3.exe", "rust_w3.exe"), "w4": ("plain_w4.exe", "rust_w4.exe")}
SAME = {"w1": "plain_w1_group_local.exe", "w2": "plain_w2_same.exe", "w4": "plain_w4_same.exe"}
pat = re.compile(r"exe=(\S+) exit=(\d+) out=\[\S+ ms=([\d.]+) peak_kb=(\d+) checksum=(\d+)\]")
for w in [x for x in ("w1", "w2", "w3", "w4") if (RUNS / f"{x}{SUF}.txt").exists()]:
    d, bad, rounds = {}, [], set()
    lines = (RUNS / f"{w}{SUF}.txt").read_text().splitlines()
    assert lines[-1] == "DONE", w
    for ln in lines:
        if not ln.startswith("round="): continue
        m = pat.search(ln)
        if not m: bad.append(ln); continue
        exe, code, ms, kb, cs = m.group(1), int(m.group(2)), float(m.group(3)), int(m.group(4)), int(m.group(5))
        d.setdefault(exe, []).append((ms, kb, cs, code))
    med = {e: (statistics.median(x[0] for x in v), statistics.median(x[1] for x in v)) for e, v in d.items()}
    c, r = med[BASE[w][0]], med[BASE[w][1]]
    s = med.get(SAME.get(w, ""), None)
    print(f"== {w}{SUF}  (known checksum {KNOWN[w]}; unparsed lines: {len(bad)})")
    print("program | runs | median ms | spread | x plain C | x safe Rust | x same-layout C | median peak KB | mem x plain C | checksum ok")
    for e, v in d.items():
        ms = [x[0] for x in v]; ok = all(x[2] == KNOWN[w] and x[3] == 0 for x in v)
        sp = (max(ms) - min(ms)) / min(ms)
        same = f"{med[e][0] / s[0]:.3f}" if s else "-"
        print(f"{e} | {len(v)} | {med[e][0]:.3f} | {sp:.1%} | {med[e][0] / c[0]:.3f} | {med[e][0] / r[0]:.3f} | {same} | {med[e][1]:.0f} | {med[e][1] / c[1]:.3f} | {ok}")
