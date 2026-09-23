# Pass 0a — Findings coverage of 0.9.9_Hardened_1

**Question (owner, 2026-09-23):** is every one of the 214 research findings (`tasks/audit/FINDINGS.md`,
F-001–F-214) accounted for in 0.9.9_Hardened_1, and does the text really do what the resolution
claims?

**Answer: yes — all 214 are accounted for.** 170 are resolved by a language change or a new rule, 18
are compiler defects against a rule that stands or was clarified, 12 are resolved by a test or CI
obligation now in the text, 13 are outside a language specification (process, repository hygiene)
with the reason recorded, and 1 was withdrawn during the research. Appendix G of the spec lists every
finding with its resolution and the rules that carry it.

## How it was checked

1. **Mapping.** `tools/findings_map.py` holds one entry per finding: kind, resolution, and the rule ids
   that carry it. `tools/check_findings.py` fails if any of F-001–F-214 is missing, if an extra id
   appears, or if a cited rule id is not defined in the parts. Result: 214 mapped, 0 problems.
   Appendix G (`parts/p25-appx-g.md`) is generated from the same data.
2. **Rule text against resolution.** Every finding's resolution was written while reading the rule it
   cites. For the 61 findings of severity S1 or S2, the opening text of every cited rule (91 rows) was
   extracted into `passes/_s1s2_evidence.md` and read against the resolution; each rule does what
   the resolution says.
3. **Gaps found by the audit and fixed in H1.** Writing the map exposed places where the text did not
   yet do what a finding needed. Each was fixed before the map was accepted:

   | Finding | Gap in the draft | Fix in H1 |
   |---|---|---|
   | F-023 | privacy violations still shared `E1020` | `E1052` for visibility; `E1020` only for a duplicate name (`[MOD-2]`) |
   | F-145 | `E9010` still had two meanings | `E9001` invalid manifest, `E9010` unknown lint, `E9041` unhonourable float attribute (`[MAN-1]`, `[MAN-3]`, `[TYP-9c]`) |
   | F-154 | shape N8 still asked for a call-site mode | N8 rewritten; call sites never write modes (`[FN-2a]`) |
   | F-160 | `ref e` expressions were used but not in the grammar | `[GRM-36]`, `E0111` |
   | F-203 | nothing let a generic struct's method name its own type | `[STR-7]` |
   | F-204 | `Box[T]` was not read through | `[TYP-14]` extended |
   | F-205 | explicit callable bounds unspecified | `[CLO-14]` |
   | F-210 | nominal interfaces were implied, not stated | `[TYP-40]` |
   | — | `[CT-2]` introduced `E6003` though 0.9.8 already had `E6010` for the same thing | `E6010` reused |

4. **Supporting checks run on the built document** (all green at the end of this pass):
   * `tools/check_ids.py` — every rule defined once; every id not in 0.9.8 marked *(new in 0.9.9)*;
     no id marked new that 0.9.8 already used (two collisions, `FFI-43` and `IMP-1`, were caught and
     renumbered).
   * `tools/check_citations.py` — every cited rule id is defined (12 dangling citations fixed).
   * `tools/code_registry.py` — every diagnostic code the text names is in the registry (§XVII.9) and
     every registry row cites a rule that exists.
   * `tools/check_examples.py --desugar` — all 45 `ember` blocks parse with the 0.9.8-era parser once
     the constructs new in 0.9.9 are rewritten into old equivalents; without rewriting, the only
     failures are those constructs (`some`, map/set literals, comprehensions, `//`, chained
     comparisons, `comptime(e)`, `import c … as`). One real mistake (a loop at file scope in §XI.4) was
     found and fixed.

## Findings outside a language specification

| Finding | Why it is not a spec change |
|---|---|
| F-001 | audit bookkeeping; the missing index was rebuilt in `FINDINGS.md` Part D |
| F-142 | the size of a compiler source file |
| F-159, F-199, F-200 | project status and ledgers; `ember --version --matrix` (`[CLI-19]`) makes implementation status visible |
| F-164 | an engine-specific example, no longer in the language document |
| F-172, F-173, F-178 | the 0.9.8 file mixed language, compiler design and history; 0.9.9 is the language only |
| F-177 | a self-checking checklist inside the document; the checks are now tools |
| F-179 | 1.0 is defined by conformance and gates (`[CONF-1]`), not by a host migration |
| F-201, F-212 | repository documents outside the spec |
