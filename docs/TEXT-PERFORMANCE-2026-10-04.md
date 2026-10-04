# Text performance continuation — 2026-10-04

> **Superseded 2026-10-04** by the autopilot session (`docs/HANDOFF.md`, start-here): its batch is adopted and published with D-499 to D-501. Kept for its evidence.

**Current status: PAUSED at the owner's request.** The later full benchmark and
regression work is recorded in
[BENCHMARK-PUBLICATION-PAUSE-2026-10-04.md](BENCHMARK-PUBLICATION-PAUSE-2026-10-04.md).
Its confirmed slowdowns, corrected GCC storage comparison and remaining work
supersede the publication/next-action status below. No changes were committed or pushed.

The owner resumed the paused work. The focused `t6_char_indices` comparison now
has actual measurements under MSVC, LLVM clang and GCC. The general decoder
inline policy and the MSVC checked append fast path are implemented in the
working tree. They are uncommitted. The text performance audit remains **OPEN**
because short dynamic reductions still pay avoidable guard setup costs.

## Whole-program comparison

Each entry is the median of six cycle medians of paired Ember/C ratios; each
cycle has eleven interleaved pairs after warmups. Both programs print exactly
`86999919066560` and have empty stderr. The work, compiler family, release flags,
input bytes and terminal allocation lifetimes match the existing README twins.
A matched normal-return worker that includes explicit buffer cleanup is still
pending and is a separate diagnostic.
These focused diagnostics are separate from the final README matrix.

| Shape | MSVC | LLVM clang | GCC |
|---|---:|---:|---:|
| Resumed progress/direct-view implementation | 1.186 | 0.917 | 0.989 |
| Force the existing typed decoder inline | 1.006 | 0.909 | 0.985 |
| Also force checked append inline on every compiler — rejected | 0.957 | 0.914 | 1.021 |
| Selected policy: MSVC append fast path; clang/GCC retain the existing append call | **0.958** | **0.906** | **0.985** |

The selected policy's paired cycle-bootstrap 95% intervals are respectively
0.953–0.961, 0.903–0.923 and 0.981–0.988. Do not interpret differences between
separate runs as isolated gains on clang/GCC. The MSVC decoder call disappeared
from the actual hot-loop assembly; its original per-character calls remain in
the preserved baseline assembly. MSVC's checked fallback still has the overflow
branch and panic. Its remaining large-input gap disappeared when the existing
checked byte append was exposed at the call site. No capacity, borrow, overflow,
UTF-8, retain/drop or runtime initialization/shutdown contract was removed.

Windows timings use verified P-core affinity `0xC03C03`. GCC uses WSL guest CPU 0;
host P-core affinity is unverified, so do not compare GCC absolute times with
Windows. All four complete whole-program timing windows passed fresh mains
checks and exact-window Windows Kernel-Power event 105 audits, with zero events.
All native queues were serial and their Windows Jobs/Linux groups drained.

The pre-pause six Windows object/assembly commands were authenticated and reused.
Only the missing four GCC commands were resumed in a fresh persistent guest root.
The cancelled pre-pause receipt remains cancelled. New runtime/link commands use
the production policy. Actual clang default linking succeeded with lld-link and
`libcmt`/`oldnames`; no automatic `-rtlib` override was added. This does not establish
the provider of every possible compiler helper. Captured full environment digests
differ from the old object builds; exact complete environment equality is not
claimed. Explicit source/driver/subtool pins, flags and matched family linking
were recorded; this is not a complete SDK/library-byte inventory.

Fresh current-driver C agrees with each measured policy input after exactly one
String append helper redirect. GCC emitted on Windows additionally changes the
exact benchmark filename in its `#line` and panic-location literals from the
mounted Linux path to its Windows path. That relocation is explicitly recorded;
the original failed byte-equality check is preserved. It is not a new native
Linux generation claim. The separate correctness run rebuilt the actual Linux
compiler from a fresh current-source snapshot and passed its GCC native cases.

## Short dynamic reductions — still open

The actual emitted `@noinline reduce(str, initial)` function was extracted from
current native conformance C. Its retained checked-loop control changes only the
initial entry to bypass the guard. A separate handwritten C decoder is the twin.
All three kernels use the same dynamic parameters and return checksums verified
against an independent scalar/byte-offset calculation. Each sample repeats one
million calls for 0/1/7 bytes or 200,000 for 60 bytes. Allocation, process startup
and output occur outside the native monotonic timer. These are kernel diagnostics,
not README process times. The same six-cycle/eleven-pair protocol applies.

| Input bytes | MSVC guarded / checked / C, ns per call | clang guarded / checked / C | GCC guarded / checked / C |
|---|---:|---:|---:|
| 0 | 0.872 / 0.789 / 0.452 | 0.584 / 0.513 / 0.387 | 0.623 / 0.386 / 0.338 |
| 1 | 2.239 / 1.164 / 0.779 | 2.093 / 0.790 / 0.660 | 2.116 / 0.782 / 0.580 |
| 7, mixed Unicode | 3.224 / 2.444 / 2.133 | 2.447 / 1.798 / 1.872 | 2.547 / 1.989 / 1.951 |
| 60, mixed Unicode | 30.790 / 34.138 / 28.657 | 24.535 / 24.493 / 27.392 | 27.356 / 26.241 / 26.962 |

D-498 is **OPEN**. The division guard is unnecessary work when the original
checked loop is cheaper. This is removable optimizer overhead, not a mandatory
safety-cost verdict. The 60-byte MSVC/GCC gaps to C also remain open. Next, price
a general profitability gate before division, including empty input, ASCII and
Unicode lengths around its crossover. If adopted, the independent actual-guard
verifier must validate that exact additional fallback condition and reject its
mutation. Retain the original fallback, effect ordering, negative-initial-total
room arithmetic and zero-count-before-accumulator-read guarantees. The earlier
checked-product guard proposal remains an unimplemented alternative.

The first short-input monitor stopped during GCC on a Windows/WSL replacement
sharing race. Its raw results and failure remain preserved. Only GCC short
timings were repeated using immutable timestamped power receipts, with freshness
checks and a complete successful power-monitor/event window. Windows short
results were retained; their complete event audit passed. Do not use the first
GCC short run as the current result.

## Correctness and remaining validation

The current compiler builds on Windows and Linux. Nine backend tests pass.
Fifty-four focused native cases pass: six cases × three profiles × three compiler
families. These include first overflow, effects before overflow, signed starts,
Unicode/NUL/empty views, field exclusivity and the new append evaluation/growth
case. The exhaustive every-Unicode-scalar guard-page decoder oracle passes under
MSVC, clang and GCC, for both signed and native-width cursor types.

The new append regression was RED before the StringPush redirect: its generated C
lacked the fast-path helper. Current native C contains the helper and passes its
receiver/argument evaluation, growth, empty append and embedded-NUL checks in all
nine selectors. A copied test oracle was initially decoded as Windows cp1252;
the actual program output was correct. That harness failure, its explicit UTF-8
correction and the five reused completed MSVC results are preserved separately.

The full workspace, annotation and eleven-gate passes in the pause checkpoint
are historical and precede these production changes. Do not label them current.
Current eleven gates and byte-identical Appendix generation passed on a frozen
copy after the integrated correctness checks. The deterministic-math gate first
hit the sandbox's global runtime-cache write restriction; its failed record is
preserved. A fresh task-owned `EMBER_CACHE` allowed the remaining checks to pass,
with seven completed gates retained. This status-only documentation update
follows that frozen gate checkpoint; no source code changed afterwards.
Before committing/pushing, finish relevant performance work and the required
current complete validation. The full README matrix runs once after all final
changes; its published values and generated benchmark assets are unchanged.
The other three close rows and the wider slowdown inventory remain OPEN.

## Evidence locations

All immutable receipts, raw samples, compiler streams and failed controls remain
under `C:\Users\ism19\AppData\Local\Temp\ember-codex-autopilot-20261003`:

- `t6-resume-20261004-01`: explicit pause-document transition, reused Windows
  bindings, GCC continuation, default links, baseline samples and power audit.
- `t6-inline-diagnostic-01`, `t6-extend-diagnostic-01`, `t6-append-policy-01`:
  frozen candidates, source changes, actual assembly, oracles, paired samples,
  cleanup and event audits. The middle append candidate is rejected on GCC.
- `t6-integrated-validation-01`, `t6-integrated-validation-02`,
  `t6-encoding-root-cause.json`: failed oracle copy and verified continuation.
  Fresh GCC source is `/home/ism19/ember-t6-integrated-validation-02`.
- `t6-current-emission-check-01/source-path-reconciliation.json`: exact current
  emitted-C relation, including the explicit GCC diagnostic filename relocation.
- `t6-short-inputs-01`, `t6-short-gcc-followup-01`: short kernels, raw samples,
  exact source bindings, monitor failure, valid GCC continuation and power audits.
- `t6-current-gates-01`: frozen current sources, seven retained gate passes,
  preserved native-cache failure, successful owned-cache continuation, all eleven
  gate results and byte-identical Appendix.

Preserve every prior evidence directory and the owner's untracked `build/`.
