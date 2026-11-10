# Cutting the spec folder's bloat without losing anything (plan, 2026-11-10, not started)

The owner: "there is a LOT of repetition and bloat and I think it is one of the reasons problems are
harder to find ... it would be nice to have changes that happened over a period of time but not the
whole specs written again and again".

## What is there now (measured)

| What | Count | Size |
|---|---|---|
| The spec itself (`tasks/spec-0.9.9/parts`) | 1 | 1.5 MB, 18,234 lines |
| Frozen copies of `parts` (`parts-h1` … `parts-0.9.20-h10`) | 108 folders | 84 MB |
| Full assembled copies (`docs/spec-source/Ember_v*_Hardened_*.md`) | 132 files | 92 MB |
| The whole repository's git store, every copy above included | | 48 MB |

- **The frozen folders are pure duplicates of git history.** Each one is exactly `parts/` as it
  stood at its cut's commit; checked byte for byte for `0.9.20-h5`, `0.9.20-h10`, `0.9.15-h11`,
  `0.9.10-h20` and `h30`. The full copies are likewise files git already holds.
- **Little reads them.** CI reads none. Three spec tools read old copies:
  - `appx_h.py` reads 59 frozen folders to build Appendix H;
  - `check_ids.py` reads `Ember_v0.9.8_Hardened_3.md`;
  - `p17a_0915.py` reads `parts-0.9.15-h1`.
- **Old review files** cite old copies by line. Those stay readable through git.

Inside the spec (18,234 lines):

| What | Size | Note |
|---|---|---|
| Appendix H, the change log | 3,629 lines (20%) | history inside the spec |
| Appendix I, the rule index | 1,450 lines (8%) | generated; every rule's first line a second time |
| Version tags on rules | 1,251 | e.g. `RC-4` carries four: changed in 0.9.9, 0.9.10, 0.9.15, 0.9.20 |
| Text repeated between rules | 400 pairs of rules share 10-word runs | the worst pairs restate whole paragraphs |

The worst pairs, by runs they share:
- `DSP-4`~`TYP-44`: 101
- `ALC-1`~`TYP-46`: 96
- `IFC-6`~`TYP-17`: 74
- `GRM-42`~`OWN-13`: 71
- `CLO-16`~`THR-1`: 71
- `LT-45`~`TYP-15`: 66
- `COST-6`~`FFI-53`: 57

A fact said in two places drifts. Today's review found one such drift: `[FFI-6]` still lists the
`link` option `[FFI-49]` removed.

No sentence of 14 words or more is repeated word for word. The repetition is paragraphs restated
with small changes, which is the kind that drifts.

## The plan

### Stage A — DONE 2026-11-10 the owner's way, not as written below

The owner chose: "keep the first copy and then consecutively create diff files for each cut and keep the
final spec that is being use". Done: `docs/spec-source/history/` (the first copy, one diff per cut,
checksums, the folders' layout), `tools/spec_history.py`, the 131 old files and 108 folders deleted, the
three tools moved over, every gate unchanged; see the top block of `docs/HANDOFF.md`. The git-tag design
that follows was not used.

### Stage A as first proposed (git tags; not used)

1. **Tag every cut.** Each cut gets a git tag (`spec/0.9.20-h10`, …) on the commit that made it,
   found from git's log. The tags stay local until the owner says to push.
2. **Two small scripts.**
   - `spec_at.py 0.9.20-h5` writes any old cut into a temporary folder from its tag.
   - `spec_diff.py 0.9.20-h5 0.9.20-h10` prints what changed, rule by rule: the word-level diff used
     for today's reviews, widened to ids like `BLD-FFI-3` and to tables. This gives the owner's
     "changes over a period of time" between any two cuts: 0.9.20-h5 to h10 is 15 rules.
3. **Point the three tools at git.** `appx_h.py`, `check_ids.py` and `p17a_0915.py` read old cuts
   through `spec_at.py` instead of the folders.
4. **Delete** the 108 frozen folders and the 131 old full copies. Keep `parts/` and the pinned
   development target. The working tree loses 176 MB; git keeps every copy in history.
5. **New cut recipe** for whoever cuts: a commit, a tag and a change-log entry; no frozen folder and
   no full copy except the pinned target. Each cut then costs a few KB, not 3.4 MB.

**Pilot first:** the 0.9.20 series only (10 folders, 10 copies). Passing means:
- every regenerated copy is byte-identical to the deleted one;
- Appendix H regenerates byte-identical;
- every spec gate and spec-tied test is green.

The rest of the series follow only after that.

### Stage B: a shorter spec, same meaning (a new cut, as "only additions in place" requires)

1. **Appendix H moves out** to `CHANGES.md`, built from the tags by `spec_diff.py`, with each cut's
   reason from its commit message. The spec keeps a one-page "what changed in 0.9.20".
2. **Appendix I moves out** to a generated `RULES.md`, so a search finds each rule once.
3. **Version tags.** Only "new in 0.9.20" and "changed in 0.9.20" stay, since they show what to build
   now. Older history comes from `spec_diff.py`.
4. **Each fact said once.** The owning rule keeps the text, the other rule cites it. The top three
   pairs go first as a pilot, each checked so that the merged text keeps every promise of both. A
   new check reports any new 10-word run shared by two rules, so a copy cannot come back unseen.

Messages are left inside the rules for now: about 24,700 words, 15% of the rule text, since they
show what each rule refuses.

**Expected:**
- the spec goes from 18,234 lines to about 13,000 (-28%) from steps 1 and 2 alone, before step 4;
- no rule changes meaning, which `spec_diff.py` of the new cut against `0.9.20-h10` proves: it shows
  only removed tags, moved appendices and merged passages.

## Cost and order

Solo, scripts only, no agents. The Stage A pilot is about an hour of work. Nothing is pushed, and
nothing outside these folders is touched. The other agent's cut recipe changes, so the owner should
tell it, or let me write the new recipe into `docs/HANDOFF.md`.
