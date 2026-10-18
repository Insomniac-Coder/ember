# Review of 2026-10-18

Nine read-only agents (Opus for six parts written solo: ADR-114, ADR-120, ADR-127, EXC-10/ADR-158,
ADR-134/D-467, and a `T` vs `T.Name` sweep of the type checker; Sonnet for D-202/D-218, D-201 and
D-355/D-358), three at a time; each finding then checked with a probe program. Fixes: ADR-201.
`phase1-findings.json` holds every agent's summary, what it checked and its findings.

| Finding | Part | Verdict | Defect | Evidence |
|---|---|---|---|---|
| F01 | adr127-temporaries-end | confirmed | D-672 | probe printed `defer` before `drop 1` on an early return |
| F02 | adr127-temporaries-end | confirmed | D-673 | `width(hold(text()))` was E3060 |
| F03 | typeck-param-vs-assoc | confirmed | D-674 | E2040 at the `extend` and at `+` |
| F04 | typeck-param-vs-assoc | confirmed | D-675 | E2020 for `T.Output` and `I.Item` |
| F05 | typeck-param-vs-assoc | confirmed | D-676 | E2040 at the f-string hole |
| F06 | d202-d218-field-access | refuted | - | the write panics with an exclusivity violation (debug and release): no use after free |
| F07 | adr134-d467-view-borrows | confirmed | D-677 | gcc: "incompatible pointer type", exit 1 |
| F08 | adr134-d467-view-borrows | confirmed | D-678 | E3060 at `self.view()` |
| F09 | d355-d358-void-and-nesting | confirmed | D-679 | the C compiler failed on `&0` |
| F10 | d355-d358-void-and-nesting | refuted | - | 200 nested `Box`es printed with Debug compile and run on clang and MSVC |
| F11 | d355-d358-void-and-nesting | confirmed | D-680 | MSVC C1061 "blocks nested too deeply" at 200 levels |

Clean (no findings): adr120-progress-reduction, adr114-temporary-owners, exc10-outer-loop-access,
d201-string-vs-bytes.
