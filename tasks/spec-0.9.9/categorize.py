import re, sys
SP=r"C:\Users\ism19\AppData\Local\Temp\claude\C--Users-ism19-Code\a8b56c7a-18b6-4c46-86aa-941c239e579e\scratchpad"
rows={}
for l in open(SP+r"\titles.tsv",encoding="utf-8").read().split("\n"):
    fid,kind,sev,t=l.split("\t")
    rows[int(fid[2:])]=[kind,sev,t]

CATS=[
 ("C1","Memory safety and soundness","compiler + spec; owner rulings for the spec items",
  [78,79,104,130,147,190,191]),
 ("C2","Miscompiles and undefined behaviour in the generated C","compiler (`DEFECTS.md`)",
  [102,139]),
 ("C3","Accepted but silently ignored or broken","compiler (`DEFECTS.md`)",
  [4,12,123,161,184,192]),
 ("C4","Compiler crashes and hangs","compiler (`DEFECTS.md`)",
  [136,185]),
 ("C5","Core language features missing or wrong in the compiler","compiler (`DEFECTS.md`)",
  [2,3,29,58,73,137,148,160,180,181,183,196,197,203,204,205,208]),
 ("C6","Standard library to build","`std/` (`DEFECTS.md` where a rule names the API)",
  [77,80,88,114,162,182,189,211]),
 ("C7","Diagnostics and error pages","compiler diagnostics + `docs/errors/`",
  [5,6,8,9,10,11,15,23,82,83,98,149,150,151,153,169,186,187,188,195,198,206]),
 ("C8","Performance","compiler backend + runtime; needs the perf suite first",
  [138,140,141,193,194]),
 ("C9","Language design — Python feel","owner decision (`OWNER-QUEUE.md`), then a language version",
  [14,31,38,48,49,54,55,60,65,68,74,93,113,155,209,210]),
 ("C10","Language design — remove Rust-style friction / simplify","owner decision, then a language version",
  [18,24,25,36,40,41,45,46,61,67,70,87,95,168]),
 ("C11","Language design — other semantic choices","owner decision",
  [19,37,64,90,107,133]),
 ("C12","Spec contradictions that need an owner ruling","`spec-errata.md`; nobody moves until ruled",
  [30,32,33,43,47,52,62,81,86,89,101,108,112,122,143,157,174,176]),
 ("C13","Spec gaps — the text is silent","`OWNER-QUEUE.md`",
  [42,44,56,59,63,66,92,94,106,109,110,119,126,128,134,175,213]),
 ("C14","Spec editorial fixes — no change of meaning","hardening-class edits (`spec-amendments.md`)",
  [7,16,17,20,21,22,27,28,34,35,39,50,51,53,57,71,72,75,76,84,91,96,97,99,105,111,116,117,118,120,124,125,129,131,132,135,145,152,154,158,163,164,165,167,170,172,177]),
 ("C15","Tooling, tests and CI gates","`tools/`, `tests/`, CI",
  [1,13,26,85,100,142,144,146,171,207,214]),
 ("C16","Docs and project process","docs + process (owner for the process items)",
  [103,115,121,127,156,159,166,173,178,179,199,200,201,202,212]),
 ("C17","Withdrawn","—",[69]),
]
seen={}
for c in CATS:
    for i in c[3]:
        if i in seen: print("DUP",i,seen[i],c[0]); 
        seen[i]=c[0]
missing=[i for i in rows if i not in seen]; extra=[i for i in seen if i not in rows]
print("missing",missing,"extra",extra,"total",len(seen))

OVR={1:"`tasks/audit/TASKS.md` (the audit's index) was never pushed",
 21:"`@nopanic` listed as v2 while `@nopanic(explicit)` is a current contract",
 66:"integer `**` with a run-time negative exponent or overflow",
 77:"spec examples call APIs that do not exist (`sum`, `Array[f32]([…])`, `retain`)",
 81:"a borrowed class-handle parameter cannot be written through (`E3023`) — text and example disagree",
 98:"`@realtime` forbids `x / n` via `@nopanic(explicit)`; needs fix-its",
 99:"`@nopanic(explicit)` is missing from the attribute table",
 103:"`[TOOL-1]`–`[TOOL-4]` sit inside X.3 \"Inspection\"",
 111:"more example syntax the grammar lacks (`SoA[T].Ref`, `mut` at call sites, …)",
 120:"`unsafe(reason = …)` is not in the grammar; three annotation channels for one fact",
 143:"effect analysis is said to run both after and before monomorphisation",
 153:"N1's threshold (≤ 2 *or* ≤ ⅓ length) admits `io`→`Eq`, `min`→`main`",
 182:"`Option`/`Array`/`str` basics missing (`unwrap_or`, `pop`, `str.len`, iterating `[T; N]`)",
 159:"Phase 1 is recorded complete but Parts II–VI features are missing",
 43:"which interface `<` uses is unclear, and `Ord` excludes floats",
 78:"a `Sync` class may have no dynamic exclusivity — the text contradicts itself (data race)",
 104:"`[THR-1]` admits a `Sync` class with a plain mutable scalar field (data race)",
 12:"`println` of a `String`/f-string/tuple/`Option`/struct emits C that does not compile",
 69:"(withdrawn: unmarked callbacks already accept local views — see F-168)",
}
def title(i):
    t=OVR.get(i,rows[i][2]).strip()
    return t.rstrip(".:").rstrip()
out=[]
out.append("<!-- CATEGORY-INDEX-BEGIN (generated; regenerate rather than hand-edit) -->")
out.append("## Findings by category\n")
out.append("Every finding is in exactly one category, chosen by **what fixing it takes**")
out.append("(who moves, and where the change lands). Kind and severity are the entry's own.")
out.append("The full evidence for each id is in its entry under *Findings* below.\n")
out.append("| # | Category | Count | S1 | S2 | S3 | S4 | Where the fix lands |")
out.append("|---|---|---|---|---|---|---|---|")
for cid,name,where,ids in CATS:
    sv=[rows[i][1] for i in ids]
    out.append(f"| {cid} | [{name}](#{cid.lower()}) | {len(ids)} | {sv.count('S1')} | {sv.count('S2')} | {sv.count('S3')} | {sv.count('S4')} | {where} |")
tot=[rows[i][1] for c in CATS for i in c[3]]
out.append(f"| | **Total** | **{len(tot)}** | {tot.count('S1')} | {tot.count('S2')} | {tot.count('S3')} | {tot.count('S4')} | |\n")
out.append("The 39 audit defects of Part D sit in the same scheme: SAFE-1..6 and RC-1..3 → C1 (RC are leaks, not")
out.append("unsafety, but they live in the same drop/move code); UB-1..5 → C2; CG-1, CG-2, FE-3 → C3; PERF-2 → C4;")
out.append("FE-1, FE-2 → C5; DIAG-1..4 → C7.\n")
out.append("**Suggested order of work:** C1 → C2 → C3 → C4 (correctness; no owner input needed except C1's")
out.append("spec items) → C15's gates (so the classes above cannot come back) → C5 + C6 (the first-programs")
out.append("milestone) → C7 → C8 once the perf suite exists. C9–C13 are one owner session: a batch of")
out.append("rulings. C14 can be done any time as one hardening. C16 alongside.\n")
for cid,name,where,ids in CATS:
    out.append(f"### {cid}")
    out.append(f"**{name}** — {len(ids)} · lands in: {where}\n")
    out.append("| Id | Sev | Kind | Finding |")
    out.append("|---|---|---|---|")
    order={"S1":0,"S2":1,"S3":2,"S4":3,"-":4}
    for i in sorted(ids,key=lambda i:(order.get(rows[i][1],5),i)):
        out.append(f"| F-{i:03d} | {rows[i][1]} | {rows[i][0]} | {title(i)} |")
    out.append("")
out.append("<!-- CATEGORY-INDEX-END -->\n")
block="\n".join(out)
p=r"C:\Users\ism19\Code\ember\tasks\audit\FINDINGS.md"
s=open(p,encoding="utf-8").read()
if "<!-- CATEGORY-INDEX-BEGIN" in s:
    a=s.index("<!-- CATEGORY-INDEX-BEGIN"); b=s.index("<!-- CATEGORY-INDEX-END -->")+len("<!-- CATEGORY-INDEX-END -->\n")
    s=s[:a]+block+s[b:]
else:
    anchor="---\n\n## Findings\n"
    assert s.count(anchor)==1
    s=s.replace(anchor, block+"\n"+anchor)
open(p,"w",encoding="utf-8").write(s)
print("written")
for cid,name,where,ids in CATS:
    sv=[rows[i][1] for i in ids]; print(cid,len(ids),name, "S1=%d S2=%d S3=%d S4=%d"%(sv.count('S1'),sv.count('S2'),sv.count('S3'),sv.count('S4')))
