# The owner's answers to the 85% helpers' questions (2026-10-18/19)

Each was put to the owner one at a time; the helpers' own wording is in `A-notes.md`, `B-notes.md`
and `D-notes.md`. **These are the owner's decisions: apply them as written, never re-decide them.**
The rules for applying them (the owner, 2026-10-19):
* Spec questions are the owner's: explain each one at a time, starting with the issue, with a real
  example; never decide one yourself. Bugs are fixed, never asked. Something that only waits on
  unbuilt work is recorded as waiting, not raised as a question.

## Status and order of work (2026-10-19; update this table as each is done)

| # | Decision | What is left | Size | Status |
|---|---|---|---|---|
| B1 | `@ffi(destroys(f))` on the foreign type | nothing: built that way | — | done |
| A5 | `ReloadError` as A built it (`type_name: String`, `instance: u64`, …) | nothing: spec `[HR-38]` states them (ODR-121) | small | done 2026-10-19 (H13, ADR-205) |
| B5 | not a question: `[FFI-16]`/`[ERR-6]` wait for C header import | nothing until C header import | — | waiting |
| B4 | fix `HOOK(7)` on a `static` of function type (D-692) | nothing | small | done 2026-10-19 |
| A2 | a `release` (formerly `shipping`) build ignores `[build] reload`, printing `note: hot reload is off in release builds` | nothing (ODR-118; `E9033` retired) | small | done 2026-10-19 (H13, ADR-205) |
| A3 | in a hot-reload build `@inline` is not applied to a reloadable function; other builds inline | nothing (ODR-119) | small | done 2026-10-19 (H13, ADR-205) |
| A4 | an upgraded object runs no user `drop` | nothing (ODR-120) | small | done 2026-10-19 (H13, ADR-205) |
| B6 | `[ERR-6]`'s spelling `@ffi(status=E, ok=X, error=Name)`, error named in the label | code when the status feature is built (waits for C header import) | small | spec done 2026-10-19 (ODR-124); code waiting |
| B2 | `E5068` "one live foreign object adopted twice" at compile time (`[FFI-36b]`; `E5059` was taken) | nothing (ODR-123) | medium | done 2026-10-19 (H13, ADR-205) |
| B7 | an `inout_count` call gives back C's own result first, then every pointer output in declaration order | nothing for a plain C result (ODR-125); a result with a contract is still `E0900` | medium | done 2026-10-19 (H13, ADR-205) |
| A6 | references in tuples, `Box` contents and closure captures are searched when an object moves | nothing (ODR-126); `dyn` values are still not followed | medium | done 2026-10-19 (H13, ADR-205) |
| B3 | `Retained.ptr()` is a typed pointer; a new pointer type for class objects | spec `[FFI-23]`; code; tests | medium | **next** |
| A1 | at `std.hot.checkpoint()` the live values below it are rebuilt in the new shape (stack maps), so layout changes apply live | spec `[HR-3]`/`[HR-33]`; stack maps; the swap; tests | large | after B3 |
| P1 | **Profiles renamed** (owner, 2026-10-19): `debug` / `development` / `release`. The old `release` (opt 2, line info, symbols, backtraces, thin LTO) is now `development`; the old `shipping` (opt 3, stripped, full LTO) is now `release`, the build users get. Hot reload is admitted in `debug` and `development`. | spec `[PRF-1]`–`[PRF-3]`, `[MAN-7]`, examples; compiler `Profile`; | ~1100 tests' `#$ profiles:`; bench and tools scripts (benchmarks keep their settings: old `release` = new `development`) | large | done 2026-10-19 (H13, ADR-205); helper C's tests are mapped at its merge |

**Approvals (owner, 2026-10-19):** A2's wording approved as shown (now naming `development`/`release`
after P1); "the rest of the decisions also look fine": the other decisions' spec wording need not be
shown one by one. The owner never asked to see B3's or A1's design before building (corrected
2026-10-19: an earlier version of this file said so; that was Claude's own step, not the owner's).

## Helper B (C interop 1)

1. Destructor spelling: (a) `@ffi(destroys(f))` on the foreign type, as B built.
2. Adopting one object twice at compile time: (a) a new code, E5059 "`p` is adopted twice", added to the spec; then build the check.
3. Retained.ptr() type: (b) a typed pointer; needs a new pointer type for class objects. Owner asked about speed: none (types are compile-time only).
4. (owner) of course fix it — bugs are not questions.
5. NOT a question (owner): FFI-16/ERR-6 depend on C header import (overlays); record them as waiting on it. My error to relay it.
6. Status-feature naming/spelling (ERR-6 vs FFI-16): ERR-6 version chosen: label `@ffi(status=VkResult, ok=VK_SUCCESS)`, error type `VkError`. Follow-up asked: how the error name is formed for other enums ((a) fixed rule strip Result/Status + Error, (b) label names it error=...).
   Follow-up (naming rule for other enums), owner explained-to and ANSWERED: (b) the label names the error type: `@ffi(status=XrResult, ok=XR_SUCCESS, error=XrError)`.
7. inout_count on a C function with its own (non-status) result: (a) GIVE BACK BOTH, e.g. `(size, ok) = GetLogicalProcessorInformation(None)` (the count from the removed pointer parameter + C's own result). Explained via the real Windows function.
   Follow-up: more than two values: C's own result first, then every pointer output (`out`, the inout count) in C declaration order, e.g. `(ok, spc, bps, free_clusters, total_clusters) = GetDiskFreeSpaceA(c"C:\\")`. Owner: "got it".


## Helper C (shared libraries, packages) — waiting for the owner

Merged 2026-10-19 (ADR-206). Its questions, in `C-notes.md` §11, triaged by the owner's rule (bugs
fixed, waiting recorded, real choices asked one at a time with real code):
* **Q1 — where a version requirement such as `geometry = "1.2"` resolves from** (a registry, git tags,
  a local directory): a real choice. Built meanwhile: `E0900`.
* **Q3 — `@export_table`'s shape** (fields bound by name or by explicit defaults; the protocol in a
  second symbol or in the table's name; `threads` on the field or on its function): a real choice.
  Built meanwhile: by name, `Name_protocol_N`, the function's own `threads`.
* **Q4 — `[ABI-1]`'s versioned module symbol against `[FFI-28]`'s export list**: a contradiction in
  the specification. Built meanwhile: nothing extra exported (the runtime ABI check only).
* Q2 (`[build] global_allocator` missing from `[MAN-8]`'s keys) is not raised now: `[ALC-3]` waits on
  the `Allocator` interface, and the key with it.


## Helper A (hot reload)

A1. HR-3 vs HR-33 (checkpoint inside Ember code): option 2, FIX UP DATA AT THE CHECKPOINT. At std.hot.checkpoint() the compiler records where each live value of the frames below the checkpoint lives (a stack map), and the swap rebuilds those values in the new shape (defaults / migrate_from), so layout changes apply live too. Owner REJECTED A's body-only reading. Needs: spec wording for HR-3/HR-33 (show the owner), then implementation (stack maps at checkpoints).
A2. reload in a shipping build: (b1) IGNORE it in shipping with a one-line note "note: hot reload is off in shipping builds" (replaces E9033; PHIL-12 unchanged). Spec change: MAN-7, PRF-2, E9033 row.
A3. @inline on a reloadable function: (c) RELOADING WINS: in hot-reload builds @inline is not applied (function stays reloadable); builds without hot reload still inline. Spec: reconcile CG-C-3a with HR-9a.
A4. user drop of a migrate_from's old instance: (a) NO user drop for upgraded objects (the object lives on in its new form); fields freed as built; correct HR-2a's wording ("running user drop code" not for migrated instances). HR-2a can then be marked done once the spec text is fixed.
A5. ReloadError parts: (a) ACCEPT A's version: `type` -> `type_name` (reserved word), String for type name/reason/message/error, u64 for instance (index among the type's live objects). Spec HR-38 to state the types and the field name.
A6. references the runtime "can enumerate" (HR-15): (b) ALSO SEARCH tuples, Box contents and closure captures: the compiler describes their insides to the runtime as for classes, so those reloads succeed instead of being refused.


## Helper D (data-oriented design)

None needed the owner (checked 2026-10-19): Q1 (`Entity` as `Handle` with `Split20`) waits for
`[HND-2]`; Q2 (what a query over an `SoA` component yields) follows `[SOA-6]`'s element proxies
when queries are built; Q3 (`run(system, args…)` without variadics) waits for `[ECS-5]`; Q4
(`ArenaSoA` in `std.collections`) is the module table's (Part XV); Q5 is defect D-690, to fix;
Q6 (BEN-6, TST-6) is scheduling, on the long tail.
