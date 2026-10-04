# Benchmark publication pause — 2026-10-04

> **Superseded 2026-10-04** by the autopilot session (`docs/HANDOFF.md`, start-here): its batch is adopted and published with D-499 to D-501. Kept for its evidence.

The owner explicitly requested: “Please pause right now and update the hand off
with details of the task you were looking into”. Work is paused. Do not resume
native execution, optimization, publication, commit or push until requested.

The task was to update the benchmarks and push the accepted text optimization
batch, while ensuring existing benchmarks did not slow down. The original focused
investigation was `t6_char_indices`, a Unicode character-index reduction. The
publication check expanded to all **52 programs × MSVC, LLVM clang and GCC**,
using **37 C and 15 C++ twins** and the unchanged benchmark inputs.

## Git and source state

`main` remains at `37f4420742bf252d8bda8a584e2b88ce1417b2b4`; the remote main was
verified at that exact commit during this turn. **Nothing was committed or pushed.**
README benchmark numbers and the three benchmark SVG assets were not published.
The pre-existing README H52 metadata correction remains in the working tree.

The uncommitted production changes remain the general progress-reduction proof
and independent verifier, String-to-str descriptor field emission/projections,
typed decoder inlining, and the MSVC-only checked append inline route. No new
production policy was adopted during the benchmark restart. The positive
candidate controls described below exist only in isolated evidence directories.
The test harness's `cores()` now honours a positive `RUST_TEST_THREADS` cap;
unset, invalid and zero values retain the previous default. This is test-only.

Preserve the owner's untracked `build/`, all uncommitted source/fixture/documents,
every temporary evidence directory and all WSL source roots. Do not reset,
delete, force-push or rewrite history. Earlier state is also documented in
[TEXT-PERFORMANCE-2026-10-04.md](TEXT-PERFORMANCE-2026-10-04.md) and the earlier
[AUTOPILOT-PAUSE-2026-10-04.md](AUTOPILOT-PAUSE-2026-10-04.md); this checkpoint
supersedes their next-action/publication status.

## Validation retained

The unchanged production inputs were authenticated against the completed prior
focused checks: **54 native cases**, **nine backend tests**, all three exhaustive
Unicode guard-page oracles, the append fixture's nine compiler/profile selectors,
and **eleven gates plus byte-identical Appendix generation**. The prior focused
receipts and source manifests are preserved, with the later test/document changes
recorded separately.

The attempted serialized MSVC full workspace run took about 90 minutes and timed
out in `the_conformance_suite_runs`. It reported no assertion failure before the
timeout but **did not complete**. It must not be labelled green. The owner then
asked to restart because conformance was taking too long. No further full local
suite was launched. The pipeline now states `focused_t6_correctness`,
`full_suite_status: INCOMPLETE`, and `ci_after_push: REQUIRED`. Exact pushed-head
CI remains mandatory; no new-head CI run exists because there was no push.

## Complete current C/C++ matrix

The full current matrix completed successfully with matching stdout and empty
runtime stderr for all **156 cells**. For `t6_char_indices`, actual Ember/C ratios
are **0.972645 MSVC**, **0.926192 clang**, and **0.990311 GCC**. These are the final
matrix's eleven-sample per-side medians; do not substitute the earlier focused
six-by-eleven diagnostic ratios (0.958/0.906/0.985).

Windows uses read-back P-core affinity `0xC03C03`. GCC uses guest CPU 0; host
performance-core affinity remains unverified, so Windows/GCC absolute times
cannot be compared. Per-sample mains checks and the immutable 0.5-second host
power monitor passed. **The new exact-window Kernel-Power event 105 audits have
not yet been run**; do not infer a completed event audit from mains checks alone.

The source snapshot used by this matrix is
`1c70cdf3d1bc0fce8f5ff99b2bd2e3cbd643b95b8eab5a91eef33deafce38584`.
The catalog/input hashes, emitter/runtime/build manifests, binaries, generated C,
commands, raw samples and output bytes are recorded in the evidence listed below.

## Previous/current regressions that prevent publication

The first screen uses six cycles of **five adjacent alternating pairs**, after
three warmup pairs. This shortened the earlier six-by-eleven plan. Only flagged
cases and unchanged-code controls received a **six-by-eleven** follow-up.
All observations remain preserved; follow-ups never replace the first screen.

MSVC follow-up against the exact parent commit:

| Program | Current/parent | Cycle bootstrap 95% interval | Status |
|---|---:|---:|---|
| `t1b_str_loop` | 1.02025 | 1.01549–1.03474 | Confirmed slowdown |
| `t1_chars` | 1.01840 | 1.00969–1.02663 | Confirmed slowdown |
| `t2_bytes` | 1.01801 | 0.99812–1.03669 | Uncertain; keep investigating |
| `t3_lines` | 1.02919 | 1.02447–1.03399 | Confirmed slowdown |
| `a14_map_gap_1024` | 1.01654 | 1.00136–1.02680 | Confirmed small slowdown |
| `b16_wrapped` | 1.00002 | 0.99357–1.00654 | Follow-up includes parity |
| `t6_char_indices` | 0.98106 | 0.97130–0.99079 | Improvement |

Three initially flagged clang arithmetic cases (`b13_wrapped`, `b14_wrapped`,
`b15_checked`) have byte-identical PE sections, including code and data. Their
follow-up intervals include parity (ratios 0.99322, 0.99780, 1.00121). This is
real variation evidence, not permission to call different text binaries equal.
The PE comparison and raw follow-ups are preserved; their event audits still
need completion before publication clearance.

### GCC filesystem correction — important

The original baseline GCC executables were built into `/mnt/c/...`, while the
current executables ran from `/home/...`. This confounded whole-process startup.
For the byte-identical empty program, apparent current/parent ratios were about
0.183 in the first screen and 0.420 in its follow-up. Those two GCC comparisons
are explicitly **INVALID_STORAGE_COMPARISON**. They must never support a speed
claim, regression clearance or publication. The GCC **C/C++ matrix remains valid**:
both of its sides ran from the current `/home` tree.

All 52 baseline executables were copied byte-for-byte into a fresh owned directory
under the baseline guest home, preserving executable mode. Hashes were verified
before/after, ext4 and the same filesystem device as current binaries were checked,
and a new map reverses exactly to the original after restoring only baseline paths.
Original maps and invalid samples remain untouched.

The replacement full 52-program GCC screen completed (six-by-five, same ext4
filesystem). It confirms only these positive intervals:

| Program | Current/parent | Cycle bootstrap 95% interval |
|---|---:|---:|
| `t3_lines` | 1.05055 | 1.02882–1.07111 |
| `t5_words` | 1.07759 | 1.06843–1.08372 |
| `t6_char_indices` | 1.06359 | 1.05099–1.07581 |

The corrected byte-identical float control is near parity (1.00164, interval
0.99940–1.00851). These three text regressions require resolution; the owner's
acceptance of earlier t6/C values does **not** waive the explicit instruction to
avoid existing-benchmark slowdowns. Correct-filesystem six-by-eleven confirmation
of the three GCC text cases remains to do if needed for the next policy decision.

## Bounded MSVC append controls completed; no adoption

Four isolated runtime policies were built and measured on six programs:
`t1b_str_loop`, `t1_chars`, `t2_bytes`, `t3_lines`, `t6_char_indices`, and
`a14_map_gap_1024`. They use the current emitter/std/profile, actual inputs,
private runtime copies, source inverse checks, prebuilt production binaries,
owned Jobs, mains monitoring and six-by-five alternating samples.

The production rebuild is a calibration, not a replacement for the measured
production binary. Normal `static inline` with the same append body was roughly
neutral. Reverting to the exported append call helps t2/t3 and some layout cases
but makes t6 about **5% slower than current**, so it is not a universal solution.

The **compact exact-HEAP path** gives candidate/current ratios:

| Program | Ratio | Bootstrap 95% interval |
|---|---:|---:|
| `t1b_str_loop` | 0.95218 | 0.94516–0.95672 |
| `t1_chars` | 0.94735 | 0.94432–0.95006 |
| `t2_bytes` | 1.01810 | 0.99805–1.03863 |
| `t3_lines` | 0.99735 | 0.98020–1.01069 |
| `t6_char_indices` | 0.98984 | 0.98682–1.00816 |
| `a14_map_gap_1024` | 0.98908 | 0.97227–1.00428 |

It is promising for the character loops but alone does not resolve line splitting.
No candidate was adopted, and candidate binaries cannot clear the original screen.

The compact helper returns for count zero. It preserves the exact existing
`PTRDIFF_MAX` length/addition predicate; invalid length/addition and growth route
through the unchanged exported append. Otherwise `needed=len+count` is proved
nonwrapping/bounded, copy begins at old len, and len becomes needed. **Do not use
a SIZE_MAX-only or capacity-only proof**: the runtime's mandatory bound is
PTRDIFF_MAX. Existing growth/allocator work was already out of line; no removal
of an inlined allocator body was demonstrated.

## Source/assembly findings and next controls

MSVC t1b/t1 add guarded reduction clones, direct view fields and append expansion.
MSVC t2/t3 have no new reduction clone; after diagnostic path reconciliation,
their C changes only direct pointer/length view extraction and append construction.
Suppressing guards alone cannot explain all four regressions. The a14 source
delta is exactly two append-route changes and one descriptor producer in its
missing-key panic-message branch; those error sites are not executed by the fixed
input, so code layout/context remains a candidate rather than proven causality.

Current GCC t5's normalized C similarly changes view extraction and append names;
on GNU the append name aliases the unchanged exported routine. Its Words iterator
has two character-next call sites. The new typed-codec forced-inlining hint versus
the parent's normal static inline hint is a candidate, not yet a measured cause.

Prepared but **not executed** at pause:

- `boundedgnu-codec-policy-control.py`: four-case original-static-inline versus
  current forced-attributes controls, including a production rebuild calibration;
  exact body/inverse checks and all binary executions on guest home/ext4.
- `run-gnu-codec-controls.py`: one owned host/guest queue with fresh power/watch
  context for prepare → build → measure. Its proposed invocation is below.
- `view-route-control.py`: isolate the descriptor route using already-built compact
  append C. Replace only a verified typed pointer/length producer pair with the
  existing aggregate `vec_as_str` route; preserve inputs, headers/runtime object,
  profile, flags and link policy. Source/helper review and owned monitor wrapper
  were still being prepared when agents were interrupted. Do not assume ready.

The eventual fix must be general (e.g. a measured compiler-specific profitability
policy), keep all safety/evaluation/overflow contracts, and be validated on ordinary
fixtures. Do not select behavior by benchmark name. Keep D-498 short-input guard
profitability OPEN; it was deferred, not solved. The eight pending C++ worker
conversions and matched normal-return cleanup worker remain separate diagnostics.

All 19 marked current assembly pairs were reviewed as **translation-unit evidence**;
linked-image transformation claims are not established. Footnote ¹ is supported.
Footnote ² needs nuance: MSVC b16 retains a 4000-iteration register loop computing
rounds 4,9,…,19999 and removes four of every five overwritten computations and
intermediate stores; it does not collapse to one final computation, nor is its
large ratio a pure SIMD-throughput result. Footnote ³ must say GCC transforms both
fib functions differently: Ember reuses fib(35/34/33/32) for fib(38), while C is
also transformed into nested loops/unrolled calls. Remove “rewrites Ember far more”
and the implication that compiled C remains plain recursion.

## Evidence and restart recipes

Publication root (**OUT**):
`C:\Users\ism19\AppData\Local\Temp\em-publish-20261004-02`.
Stage (**STAGE**): `OUT\benchmark-stage`.
Prior focused evidence (**BASE**):
`C:\Users\ism19\AppData\Local\Temp\ember-codex-autopilot-20261003`.
Current GCC source: `/home/ism19/ember-publish-20261004-02`.
Baseline GCC source: `/home/ism19/ember-publication-baseline-20261004-02`.

Principal receipts/files:

- `performance-validation.json`, `publication-source-manifest.json`: honest
  focused validation, incomplete full suite and exact source transition.
- `msvc-workspace.log`: preserved 90-minute incomplete conformance run.
- `builds-build-20261004-03/complete.json`: all 156 authenticated baseline builds.
  The first build namespace failed before native work due a captured Visual Studio
  PATH missing its tool directory; the final helpers restore the checked captured
  Hostx64/x64 directory. The WSL sandbox launch failure was preserved and only the
  GCC launch resumed with escalation; the 104 Windows builds were retained.
- `benchmark-stage/readme-final-final-20261004-02-{windows,gcc}.json` and logs:
  complete matrix with raw samples/source/native/output bindings.
- `measure-final-20261004-02/complete.json`, maps and monitor health:
  original matrix, initial comparisons, marked assembly and drained ownership.
- `repeat-followup-20261004-01/`: valid Windows six-by-eleven follow-ups and
  invalid mounted-baseline GCC follow-up, all raw samples/cleanup preserved.
- `positive-screen-pe-sections.json`: actual PE code/data comparison.
- `gcc-storage-comparison-invalid-01.json`,
  `gcc-storage-ext4-20261004-01/{normalization-receipt.json,binary-map-gcc-normalized.json}`:
  explicit invalidation, byte-copy/mode/ext4/device checks and path-only new map.
- `repeat-gcc-storage-screen-ext4-01/results/gcc/readme-regression-final-20261004-02-gcc.json`:
  the **valid** complete GCC screen. Its runner is `repeat-gcc-storage-screen.py`.
- `append-policy-control-append-policy-20261004-01/` and
  `repeat-append-monitor-20261004-01/`: four-policy build/sample/calibration controls.
- `gcc-current-mirror-final-20261004-02/mirror-manifest.json`: hash-verified Windows
  copies of all 52 current GCC C/Ember/reference triples; copying is not new timing.
- `terminal-work-windows-binding-01.json`,
  `a14-current-vs-baseline-source-review-01.json`,
  `footnote-root-review-draft-01.json`: actual source/native/output/lifetime bindings
  and assembly findings. These do not claim identical native teardown.

Future GNU control command, **only after the owner resumes** and after checking
the final source/peer status and syntax of the unexecuted wrapper:

```powershell
python -B C:/Users/ism19/AppData/Local/Temp/em-publish-20261004-02/run-gnu-codec-controls.py --screen-tag final-20261004-02 --repeat-tag gnu-codec-controls-01 --codec-tag gnu-codec-01 --queues-drained
```

It needs an escalated contained WSL launch. Do not run the worker directly or use
stale stopped power receipts. Preserve existing namespaces; use fresh tags for
new attempts. Source helpers and late peer reviews may have been interrupted;
inspect their files and hashes rather than assuming any queued “ready” message
constitutes execution or a completed check.

The publication helpers `finish-evidence.py`, `gcc-storage-evidence.py`,
`followup-evidence.py` and the staged `update-readme-final.py` have explicit focused
scope and no invented full-suite PASS. This dataset requires the actual normalized
GCC result plus normalization receipt; omitting them must reject the known-invalid
default GCC screen. Full root regression dispositions, real event-105 audits,
source/lifetime/footnote review, preview/render checks and final adoption validation
are still required. No publication evidence claiming all regressions cleared exists.

## Stop state

All four executed measurement queues are finished: initial matrix/screen,
selected six-by-eleven repeats, append controls, and corrected GCC screen. Their
recorded Jobs/guest groups drained, monitor threads drained, watchdog exit codes
are zero and monitor error lists are empty. Review agents were interrupted on
the pause request. The earlier background task power poller (PID 62748, verified
exact `BASE\power-watch.py` command) was stopped. No GNU/view controls were started.
No commit/push/CI wait was started. All further task work is paused.
