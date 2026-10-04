# Autopilot pause checkpoint — 2026-10-04

> **Superseded 2026-10-04** by the autopilot session (`docs/HANDOFF.md`, start-here): its batch is adopted and published with D-499 to D-501. Kept for its evidence.

**PAUSED at the owner's explicit request. Do not resume development, compilation,
native workloads or benchmarks until the owner asks.** This checkpoint was written
after stopping the active queue and confirming its owned cleanup. It supersedes
older pending-work descriptions in HANDOFF, without changing historical evidence.

The owner said the work was going in loops. Root acknowledged that excessive
validation and runner preparation had delayed actual performance results. On
resume, produce a focused `t6_char_indices` comparison with MSVC, actual LLVM clang
and GCC before broadening the audit. Do not restart completed suites or expand
the harness without a concrete failure or measurement that requires it.

## Repository and owner requirements

- Repository: `C:\Users\ism19\Code\ember`, branch `main`.
- Last recorded commit: `37f4420742bf252d8bda8a584e2b88ce1417b2b4`,
  “Remove unused GCC executable sections and guard aligned allocation rounding”.
  Its five CI jobs previously passed. No new commit or push was made in this
  interrupted batch; the changes below remain uncommitted.
- Preserve the owner's untracked `build/` and every existing evidence directory.
  No reset, deletion, force push or history rewrite.
- Speed audit takes priority over adding language features. After that, finish
  Phases 1–3 and their dependencies. No new phase percentages were established
  by this batch; do not turn test counts into completion percentages.
- Every newly slower Ember/C or Ember/C++ pair needs investigation, even below
  10% or inside “close to C”. Only these five historical investigated programs
  have the owner's exemption: `a10_recursion`, `p1_read_loop`, `a16_map_text`,
  `t3_lines`, `p5_million_objects`. Reinvestigating them is welcome.
- A slowdown is not closed by noise, a compiler choice, pricing one check,
  or an unexplained label of reasonable overhead. Try safe general alternatives
  and measure the whole remaining gap before accepting required safety overhead.
- MSVC, actual LLVM clang and Linux GCC correctness and performance are required.
  Match work, flags, profile, lifetimes and cleanup with the handwritten twin.
- All compiler/native/timing queues are serial. The owner authorized parallel
  agents for coordinated source-only work. Development agents are now stopped.
- Any timing, including compile timing, requires mains power and a Windows
  Kernel-Power event 105 check covering its window. Windows timed processes use
  P-core affinity `0xC03C03`. Both GCC programs use the same WSL guest CPU 0;
  explicitly disclose that host P-core affinity is unverified and do not compare
  Windows/GCC absolute times.
- The full README benchmark matrix runs once after all final changes. Focused
  causal measurements are separate. Do not publish incomplete investigations.
  Printed ratios below 0.95 are bold green; otherwise green below 1.05, amber
  through 1.10, red above. Read `docs/AUTOPILOT.md` and the existing README scripts.

**README benchmark numbers and generated benchmark assets have not changed.**
The current README edit is H51→H52 metadata, not a new measured result.

## Implemented changes, still awaiting performance verdict

1. `compiler/ember_analysis/src/progress_reduction.rs` adds a general,
   conservatively certified progress/reduction transformation. It handles an
   immutable whole-text iterator with a private native `usize` cursor and a
   checked signed-i64 accumulation of a proven nonnegative bounded term. It
   rejects effects, aliases, malformed loops and unsupported proofs; has
   deterministic budgets; clones only certified paths; keeps original fallback,
   operation order, overflow panic and source span. An independent verifier
   reconstructs the actual guards/clone/proof and runs unconditionally in the
   driver immediately before C generation. Callable regions are refreshed.
2. `compiler/ember_codegen_c/src/lib.rs` directly uses `.ptr`/`.len` for exact
   immutable String-to-str views and internal typed native-usize iterator fields.
   Other ownership/type/arity cases retain the helper. No safety, borrow/access,
   retain/drop or runtime ABI contract was removed.
3. `compiler/ember_analysis/src/lib.rs` exports the analysis; driver `main.rs`
   invokes it and the final verifier. Driver milestone tests and backend tests
   cover actual emitted structure. Three new RNG-4 fixtures check Unicode,
   signed starts, first overflow and effects-before-overflow. Two SPN-1 fixtures
   check String descriptor fields and retained exclusivity.
4. Five test-only symbol expressions in `progress_reduction_tests.rs` now call
   the existing branding function. This fixed the branding gate without changing
   production behavior or emitted symbol strings. Original fixture bytes are
   archived; historical C generation receipts remain untouched.

Changed tracked files before this checkpoint: README, HANDOFF, analysis lib,
codegen lib, driver main and driver milestones. The progress module, its tests
and five conformance fixtures are untracked new source files. This checkpoint
adds documentation only; do not mistake an uncommitted feature for a landed one.

All four historical “close to C” investigations are **OPEN**:

| Program | Work already done | Still required |
|---|---|---|
| `t4_split` | D-480/D-486 split facts and UTF-8 checks | General progression/reduction proof, final matched measurements |
| `t6_char_indices` | D-484/D-485 range/native cursor work; current progress/direct-view changes | Current linked comparison, whole-gap investigation, short-input setup cost |
| `a00_empty` | D-492 GCC unused-section removal; math archive prototype | Whole startup gap, actual production integration/pricing |
| `a07_virtual_calls` | D-482 TLS/D-483 fresh ownership work; constructor/capacity investigations | Remaining ownership/layout/lifetime costs and final matched C++ comparisons |

The README's historical ratios for those rows were, respectively:
MSVC/clang/GCC `1.04/0.93/1.09`, `1.01/0.94/1.07`, `1.01/1.02/1.08`,
`0.97/0.95/1.10`. These are older README values, **not current measurements**.
The wider audit inventory contains 34 nonexempt programs and 50 raw slower
compiler pairs in the historical 52-program matrix. All remain obligations;
do not limit the rule to these four rows or rounded values.

## Completed correctness evidence

External evidence root (called BASE below):
`C:\Users\ism19\AppData\Local\Temp\ember-codex-autopilot-20261003`.
Guest mount: `/mnt/c/Users/ism19/AppData/Local/Temp/ember-codex-autopilot-20261003`.

- Full workspaces on the production-source checkpoint: MSVC 381, clang 381,
  GCC 383 tests passed, 34 groups each, zero failed/ignored. All owned Jobs and
  guest groups drained. These are historical full-suite passes, not fresh full
  passes after the test-only branding edit or this documentation checkpoint.
- 45 focused native cases: five new fixtures × three profiles × three compiler
  families, all passed. Meaningful failing controls and earlier real failures
  are preserved. Actual C shape, 18 analysis tests, four backend tests and two
  driver tests passed. No performance improvement is implied.
- MSVC and GCC annotations: 2,158 files, 302 directories each, zero failures,
  one serial worker; receipts, raw logs, input identities and cleanup verified.
- The first gate run stopped on five hard-coded fixture symbols. Its failed
  result and exact source copy are preserved in `text-gates-01`.
- After the five branding expressions changed: all 18 progress fixture tests
  passed under an owned Job, then all 11 repository gates and byte-equal
  generated Appendix passed on `text-gates-02`. No benchmark ran.
- The pure historical-pin reconciliation helper passed three positive checks
  on actual generation maps and three rejection controls. Two independent
  source reviews found no blocker. This is source identity evidence only.

Important exact pins:

| Evidence | SHA256 |
|---|---|
| Historical complete source manifest `text-gcc-01/manifest.json` | `f0d356aa4f2a1391f7fb46d7265c965a0868fac0811865f00134624566a4c01f` |
| Pre-pause complete source manifest `text-branding-followup-01/manifest.json` | `c4a8d5ad7fb5199ce59b95da4d8737b7f3b627216d77347f9dfb2464b553deaf` |
| Exact five-expression transition `text-branding-followup-01/transition.json` | `319499452ba4606ad582452d640b9aa323a397c39421948626fa881240f8d3f5` |
| Original fixture | `4f2c7a83d26b3cda284ac554a0a90f38bebc6cc2ae40c57448c5dcdd6baaea0b` |
| Current fixture | `c1f2b3d2b46f3a3b434666cb70c2deb080bbd3496e6f77fa921f7c793c8e66d9` |
| Progress module | `355335d4a94e885879923a47f9842f71838e31dc03b6044bf9bf6025dd389ff7` |
| Fresh fixture receipt | `9c961621c1eee62d5665e5cdd2546c5e6fb22fcc5b600e3a2114ee7da970f934` |
| Fresh gates receipt | `507aaf8794525e12f6865321b8964e810ce435721095d89e64e7af91852cd229` |
| Reconciliation helper manifest | `6c466fa9bb22b6ff4e1437b29eeeb9ac8ef5659c845bf754f66663341ca03e9e` |
| Reconciliation helper source | `e69656cc92d89e75587b1e05b2b8c38db7a8f252bcdf590a0efc08a52b5c35a5` |
| Root helper control receipt | `4a16928b24b28a4cb63dc9e7a7c7076e73e2055091db1df817f56db6e3c05c47` |

The original archived fixture is in
`text-gates-01/source/compiler/ember_analysis/src/progress_reduction_tests.rs`.
The transition is intentionally immutable and its old PENDING label is
historical; newer completed receipts supply the result. Do not rewrite it.
The helper authenticates 111 original production pins, preserves all other
110, checks the exact forward/inverse fixture change and seals 4,269 current
manifest entries. Its caller must authenticate the generation receipt first.

## Active queue stopped, actual partial result

Root launched the reviewed `text-objects-parent-03.py`; its unified exec session
23698 is now closed with exit 1 after the deliberate user pause. The result is
**partial/cancelled**, not a compiler defect and not an all-three GREEN stage.

Six Windows commands completed and their actual artifacts were rehashed:
`msvc-ember`, `msvc-reference`, `clang-ember-object`, `clang-ember-assembly`,
`clang-reference-object`, `clang-reference-assembly`. MSVC's object commands
also produce annotated assembly. All six individual owned Jobs returned 0,
did not time out and drained. Artifacts and raw streams are in `text-objects-01`.

GCC was stopped before any GCC command record was produced. Its new persistent
guest root `/home/ism19/ember-text-objects-current-01` may already exist;
preserve it. The stage cannot be blindly rerun because its directories and
Windows failed receipt already exist.

Pause controller `pause-text-object-queue-01.py` checked the exact owner argv,
private process group/session 443, original `/proc` start time and config pin,
then sent SIGTERM only to that group. Its existing supervisor killed/drained
remaining members and reaped the leader. Guest return −15, no timeout;
Windows owned Job PID 50248 returned 1, no timeout, empty after cleanup.
Root independently verified cleanup/config/log hashes and all six completed
Windows outputs. No compiler/native queue remains active.

| Pause evidence in `text-objects-01` | SHA256 |
|---|---|
| `receipt.json` (FAILED due explicit pause; six completed commands) | `8ff152f3ad3a3cc830ed0d61b9456b8fedecdc21da3a07da41bd10abeda86a0d` |
| `guest-cleanup.json` (group drained) | `7567960006857d8f9f25e054357ac3d37cdcce42d43e7ab0eef8ec16a17f65a6` |
| `guest-config.json` | `4841ae1ef081fbb571415b501d037500de00ab152f40001e563961d00c4f7fea` |

Current object runners, fully read, syntax checked and independently reviewed:

- `text-objects-worker-06.py`:
  `dc342842b6d445987aec4dcf0f181ef4dcb1d30455505bb49aed5ec4420073f6`.
- `text-objects-gcc-03.py`:
  `8c1366672128a542aad3d920814780a0c27d95cd27cb817fd5573a25c4cb8a50`.
- `text-objects-parent-03.py`:
  `a1891fc6dda4579e5f2a831fe84e2060e674e492d790c58585215cb3b7db8c31`.
- Original object-data pack `text-current-objects-source-first`, manifest
  `24637e50ca3c6b8e1982b4f12f7587f71df8113684321340e20c1ad156d69af0`.

Root read actual clang assembly: the guarded fast path has unchecked defined
accumulation, the original fallback retains `jo`/panic, and the division guard
sits outside the character loop. MSVC assembly retains decoder calls and also
has the guard outside the loop. These observations have not been priced.

## Prepared next stage, never executed

Historical actual current-feature C/header emissions are in `text-emission-01`.
They still belong to the original generation identities; fixture-only changes
do not establish fresh generation. Expected t6 output is
`86999919066560\n` (Windows stdio CRLF normalization only).

The handwritten C twin builds the same 3 MB text and runs 20 Unicode/byte-offset
passes. Its final allocation lifetime differs from a normal-return cleanup
worker; keep whole-program and matched-lifetime diagnostics distinct.

- `text-current-linked-source-first`: frozen nine baseline runtime/link recipes,
  manifest `2acbd58e13b676194327fb4d9fba3324219e1e4dd03393f802e97bc7fd2faea0`.
- `text-link-worker-source-second/worker.py`: fully read, syntax/member checked,
  SHA `9058e6e2628d2bb6a4f667e23529f1856be4f0ec8e721bfebd2858877f2e39bf`;
  pack manifest `241939e9a0d954886cc4c79cf7964d4b460b5658d784867e387b68f688dc12e4`.
- `text-link-parent-source-second`: new owned parent/stage copies with the three
  PENDING constants replaced. Manifest
  `cf5efe8099e8af034bd01a89a641f4c608129255a957f7f82437237afdd0c539`;
  parent `9ebc02b94a339a9a2c609c3b2621ddf837582a4af40250211400755f5b8826e8`;
  GCC stage `b9e9e68bd8fca0ea9eb87276b6d688dd86d98ff5972f83d6fa2e82c64f9ee6ea`.
  Root fully read the underlying parent/stage and checked the exact substitutions.
  Independent review of the original drafts found no concrete blocker; final
  constant-only review was interrupted by pause.
- `text-linked-01` has not been created. No current linked executable, native
  validation run or performance verdict exists. Intended guest root:
  `/home/ism19/ember-text-linked-current-01`.
- A focused timing-worker preparation was assigned to `text_audit`, then
  interrupted. No `text-t6-pricing-source-first` directory was found at pause.
  Do not assume that runner exists or passed review.

Link requirements already in the drafts: actual six-object binding and raw
post-drain hashes; same main/runtime/link environment; normal production flags;
default clang link with no automatic `-rtlib` override; raw separate `-###`
trace; GCC selected tool identities/capabilities; exact owned drain; preserve
failure evidence. Clang runtime provider remains UNKNOWN until actual trace
and linked inputs are reviewed. Do not invent a provider from an available
compiler-rt archive. No toolchain/runtime change has been made by this batch.

The pre-pause c4 manifest pins HANDOFF's old bytes. This checkpoint changes
HANDOFF and adds this file after all consumers stopped. Existing strict
runners therefore need an explicit documentation-only checkpoint refresh on
resume. Preserve historical manifests and fixture transition; do not silently
relax pins or relabel old full-suite evidence as current.

## Resume in this order

1. Read this checkpoint and AUTOPILOT. Check power afresh. Keep this pause in
   force until the owner explicitly resumes. Source-only agents are interrupted.
2. Finish only the four missing GCC object/assembly commands using a reviewed
   fresh owned continuation. Reuse the six completed Windows commands through
   explicit actual bindings; avoid recompiling them without a concrete need.
   Preserve the cancelled receipt and all existing roots. Do not run Parent03
   into existing files or pretend its failed receipt is GREEN.
3. Bind actual objects, link with the production policy, verify expected stdout
   and empty stderr under all three compilers. Resolve any actual link failure
   before timing. Keep source checks outside timing and every queue serial.
4. Run focused matched t6 comparisons, six interleaved cycles with at least
   eleven paired samples per cycle, warmups, raw samples, paired uncertainty,
   affinity and mains/event-window evidence. Report the measured ratios early.
   This is a diagnostic, not the final README matrix.
5. If still slower, inspect the measured cause and price safe general changes.
   The checked-multiply alternative below is one candidate, not a measured fix.
   Complete the other close rows and the wider inventory afterwards.
6. Integrate D-494/D-495 and ADR-120/121 documentation drafts accurately after
   evidence is current. Commit/push only when the required checks and the
   owner's resume authorization allow it; retain historical CI/checkpoints.

Do not repeat all completed Rust suites/annotations merely for fixture naming
or this documentation checkpoint. Repeat the relevant checks only after code
changes, a real failure or another unresolved concern. The recent bottleneck
was runner preparation and repeated full-source sealing; simplify unnecessary
repetition while preserving actual input identity and before/after checks.

## Other work already reviewed, still open

- `progress-guard-multiply-source-first` (manifest
  `b26c0744fee62b6a297ba68825734bc5499ef6a9b88d12b9e446af9f10f35707`):
  source-only plan to replace `R <= room/U` with a checked-u64 product and
  `!overflow && product <= room`. Correct room is
  `(uint64_t)INT64_MAX - (uint64_t)A` modulo 2^64, **not UINT64_MAX−A**.
  Keep zero-count/zero-upper branches before accumulator reads; reject wrapped
  product-only comparisons. Requires an independent paired product/overflow
  verifier, u128 oracle/boundary controls and short-input measurements. No
  implementation, native pricing or performance result exists.
- `text-doc-drafts-source-first`: D-494/D-495, ADR-120/121 drafts reviewed, not
  installed. Their old pending gate labels and old fixture pin need truthful
  reconciliation. Next slots D-496/ADR-122; do not claim the performance audit
  finished in these correctness records.
- `readme-cpp-worker-parity-source-first`: eight C++ worker/lifetime twin pairs
  prepared and fully source reviewed; no native compilation/measurement yet.
  Covers p2/p4 views, sum/read loops, structs and DOD SoA. Adopt only after
  actual emitted-C/native cleanup/stdout comparisons on all three compilers.
- `allocator-stats-followup-source-first`: public cumulative embedding stats
  cannot simply be disabled; preserve initialization/shutdown/observer contracts.
  Managed-unit LTO/dead-write or proven no-observer lifetime alternatives are
  source plans, unimplemented and unpriced. Existing allocator-inlining pricing
  was inconclusive/rejected, not a closed allocation gap.
- Constructor v6 measurements: six cycles on all three, mostly inconclusive;
  matched-worker C++ ratios MSVC 1.064 / clang 1.110 / GCC 1.201 remain OPEN.
  No adoption. Capacity weighted-DAG proof tests and numeric controls passed,
  but actual MIR owner/alias/store proof remains incomplete.
- Split progression/term proof remains incomplete, including the final empty
  token and done sentinel. Do not replace it with a benchmark-specific pattern.
- Math archive D-493/RT-108 prototype has substantial correctness evidence but
  unresolved whole startup gap and integration cost; not adopted. Earlier
  aligned SIZE_MAX rounding guard is in the last landed commit.

Power watcher PID 62748 was previously polling every 120 seconds. The last read
at 03:06:11 UTC reported mains and 100% battery. Its stop could not be confirmed:
`Get-CimInstance` process inspection returned Access denied, so no process was
killed. The controller's subsequent “already absent” output was not reliable
evidence. Treat the watcher as possibly still running; never treat its stale
status as permission to benchmark. Development agents and the compile queue
are confirmed stopped; no timing queue started.

All detailed raw receipts, controls, logs and older investigative packs remain
under BASE. This checkpoint is the durable route back to them, not a replacement
for reading actual pinned evidence when it is needed.
