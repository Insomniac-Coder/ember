# The owner's answers to the 85% helpers' questions (2026-10-18/19)

Each was put to the owner one at a time; the helpers' own wording is in `A-notes.md`, `B-notes.md`
and `D-notes.md`. None is applied yet: each is a change to the specification or the code that
follows the merge (ADR-204).

## Helper B (C interop 1)

1. Destructor spelling: (a) `@ffi(destroys(f))` on the foreign type, as B built.
2. Adopting one object twice at compile time: (a) a new code, E5059 "`p` is adopted twice", added to the spec; then build the check.
3. Retained.ptr() type: (b) a typed pointer; needs a new pointer type for class objects, design to be shown to the owner before building. Owner asked about speed: none (types are compile-time only).
4. (owner) of course fix it — bugs are not questions.
5. NOT a question (owner): FFI-16/ERR-6 depend on C header import (overlays); record them as waiting on it. My error to relay it.
6. Status-feature naming/spelling (ERR-6 vs FFI-16): ERR-6 version chosen: label `@ffi(status=VkResult, ok=VK_SUCCESS)`, error type `VkError`. Follow-up asked: how the error name is formed for other enums ((a) fixed rule strip Result/Status + Error, (b) label names it error=...).
   Follow-up (naming rule for other enums), owner explained-to and ANSWERED: (b) the label names the error type: `@ffi(status=XrResult, ok=XR_SUCCESS, error=XrError)`.
7. inout_count on a C function with its own (non-status) result: (a) GIVE BACK BOTH, e.g. `(size, ok) = GetLogicalProcessorInformation(None)` (the count from the removed pointer parameter + C's own result). Explained via the real Windows function.
   Follow-up: more than two values: C's own result first, then every pointer output (`out`, the inout count) in C declaration order, e.g. `(ok, spc, bps, free_clusters, total_clusters) = GetDiskFreeSpaceA(c"C:\\")`. Owner: "got it".


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
