# Gap map 09: the three stopped helper branches (AK, AJ, AI) against spec 0.9.15_Hardened_1

Read-only job. Nothing was built or run. I used `git log`, `git show`, `git diff <merge-base>..<branch>` and
`git merge-tree --trivial-merge` (a read-only trial merge that prints the result and writes nothing).
Sources: `tasks/96/helpers-stopped.md`, `tasks/95/{AK,AJ,AI}-notes.md`, `tasks/95/odr-pending.md`,
`tasks/96/redesign/JOB.md`, `tasks/96/redesign/PLAN-0.9.15-compiler.md`, the 0.9.15 and 0.9.10-h35 spec parts,
and the §H.41 change table.

## 1. Answer in one page

- **Where the merges go.** The three branches change the compiler at the old spec and each was verified
  there. This branch's `compiler/`, `std/` and `runtime/` are identical to `main` (`git diff main HEAD`
  on those paths is empty; `main` is 561143a1, this branch is design work on top). So the cheapest, safest
  place is **`main`, before the rework's first milestone (M1)**, in the order AK, AJ, AI, each with the full
  check (`scratchpad/verify_nobench.sh`) before the next. After M1 the old-spec proofs (AK's 3,435 programs
  with byte-identical C, AI's 2144/2144 under LLVM) can no longer be repeated, because outputs
  legitimately change.
- **Textual trial merge into today's tree** (`git merge-tree --trivial-merge`):
  AK: 0 conflict hunks (3 files changed on both sides, merged cleanly). AJ: 0. AK with AJ: 0.
  AI: **14 conflict hunks in 5 files** (`ember_driver/src/lib.rs` 5, `ember_codegen_c/src/lib.rs` 4,
  `ember_codegen_c/src/reload.rs` 3, `ember_driver/src/reload.rs` 1, `docs/phase-status.json` 1) and 15 more
  files changed on both sides that merged textually but need a compile check. AI's base (`da5f4101`) is
  before AE's merge (`3f2fce5b`), as the notes say.
- **What the rework keeps.** AK: nearly all of it (it changes how the compiler holds memory, not what it
  accepts), except the unfinished "flat curve" goal, which must be redone on the reworked pipeline. AJ:
  the idea and most code (a zero-sized type per function that converts to the callable type and is called
  directly), which fits CLO-1, CLO-14, TYP-5 rule 10 and the new EFF-2; but six things change or go
  (section 3). AI: the LLVM reload/library/ABI/PGO work stays; the RefCell half of ODR-275, the whole
  access-word fix D-913 (EXC-18 is removed, EXC-19 has no access words) and the 6 run-time-panic evidence
  tests go; the header sizes, the schema rows and the class exports must be redone for the new rules.
- **Biggest cross-area fact.** CLO-3 changed meaning: a parameter of type `fn(A) -> R` is now a function
  value (a dynamic call), no longer an implicit generic that is instantiated per function. Only an explicit
  `[F: fn(A) -> R]` (CLO-14) gives one instance per function. AJ's `callable_values` and the old
  per-function copy for a plain `fn` parameter (its test `CLO-3/accept_a_function_argument_is_called_directly`)
  belong to the old meaning.
- **Three owner questions** (section 7): whether ODR-278 (every named function has its own type) is part of
  0.9.15, since CLO-1 only says it for lambdas and local functions; whether a variable holding a function
  item counts as a "real variable" for CLO-2 and CLO-4; and the cdylib versus class-handle export
  contradiction that AI found, still present in FFI-28 and BLD-FFI-5a.

## 2. Sizes and overlap (measured from git)

| branch | base on main | commits | files | lines | crates touched |
|---|---|---|---|---|---|
| AK `stopped/AK-compiler-memory` | 3f2fce5b | 5 (4 + the stop commit) | 25 | +1385 -301 | analysis, ast, build, codegen_c, driver, mir, parser, typeck; tools/, bench/, .github |
| AJ `stopped/AJ-function-items` | 3f2fce5b | 4 | 33 (15 compiler/docs, 18 tests) | +1178 -123 | analysis, codegen_c, codegen_llvm, driver (tests), interp, mir, typeck, types |
| AI `stopped/AI-llvm-backend` | da5f4101 | 9 | 65 (38 compiler/std/docs, 27 tests) | +7342 -2343 | analysis, build, codegen_c, codegen_llvm, driver, hir, interp, mir, typeck, types; std/src/sync.em |

Files two branches both change (text overlap; the merge is clean for AK with AJ, see section 1):
AK and AJ: `ember_codegen_c/src/lib.rs`, `ember_mir/src/lower.rs`, `ember_typeck/src/lib.rs`,
`ember_driver/tests/milestones.rs`. AI with AK: `ember_codegen_c/src/lib.rs`, `ember_driver/src/lib.rs`,
`ember_mir/src/lower.rs`, `ember_typeck/src/lib.rs`, `milestones.rs`. AI with AJ: `ember_codegen_c/src/lib.rs`,
`ember_codegen_llvm/src/{ffi,types}.rs`, `ember_types/src/lib.rs`, `ember_mir/src/lower.rs`,
`ember_typeck/src/lib.rs`, `ember_driver/tests/{llvm,reload,milestones}.rs`. The `emit`/`emit_with`
signature in `ember_codegen_c/src/lib.rs` (HEAD lines 313 and 332; 14 positional arguments) is touched by AK
(adds `unit_sink`) and by AI (its 4 conflict hunks are there): resolve once, in the AI step.

## 3. One table per branch

Status words as the brief defines them, read for a branch change: `built` = the change is compatible with
0.9.15 as it stands; `partly` = kept but something in it must change; `remove` = the change is moot under
0.9.15; `not built` = unfinished work the new spec still needs. File lines are for HEAD (= main's compiler)
unless I say "branch".

### 3.1 AK: the compiler's memory set by the biggest module (ODR-281)

Rules named by the helper: BUD-1, BUD-2, BUD-3, BLD-1, BLD-2, BLD-7. In §H.41 only BLD-2 changed (text now
says interface hashes include "the facts of BLD-14"); the BUD rules and BLD-1/BLD-7 are unchanged. ODR-281 is
an owner decision recorded in `tasks/95/odr-pending.md`, not text in the spec.

| # | what the branch changes (crate/file) | status | under 0.9.15 | conflicts with the rework's likely edits | merge |
|---|---|---|---|---|---|
| K1 | Interface records and kept checked forms name imports at their *reach hash* (Tarjan components, no recursion): `ember_build/src/interface.rs` (schema 14 to 15, `reach_hashes`, `component_hashes`, `Write` enum; HEAD `SCHEMA_VERSION` line 50, `collect_transitive_dependencies` line 768), `ember_driver/src/incremental.rs` (schema 3 to 4, `transitive_imports` line 1195 becomes `direct_imports`) | built | Keeps. BLD-2 (changed) puts effect sets and BLD-14 facts into the interface hash: that adds fields to `ModuleInterfaceArtifact`; the reach hash is only the dependency form and is independent. The rework's schema must be at least 16 and 5. | `interface.rs` (area 4 adds the effect summary to the records and to `interface_hash`; area 7 may add codes), `incremental.rs` (Record gains summary parts). Hunks are in different functions from AK's, so a late merge is mostly clean, but the schema constants will collide. | before M1 |
| K2 | Free what no later stage reads: `Module::drop_bodies` in `ember_ast/src/lib.rs` called after the type check in `ember_driver/src/lib.rs` (`compile_once`, HEAD 4957); `lower_with` takes `&mut Program` and drops each HIR body once lowered, `FunctionIndex` replaces a linear callee search (`ember_mir/src/lower.rs`, HEAD `lower` 26, `lower_with` 39; call at driver line 5245); `Body::shrink_to_fit` | built | Keeps, with one rule for the rework: **no pass after the type check may read an AST body, and none after lowering may read an HIR body** (the `--report tcb` run keeps bodies on purpose). The new effect inference runs on MIR (`ember_analysis/src/effects.rs`), so it complies. | `ember_mir/src/lower.rs`: every lowering edit for `shares`, `weakref`, `owns` numbers, arenas touches `Builder`, and AK changed `Builder::new`'s signature and 11 `program.function(..)` call sites to `function_of`. Many small textual conflicts if merged late. | before M1 |
| K3 | Sparse initialisation facts: `SparseStates` in `ember_analysis/src/facts.rs`, `definite_init.rs` (HEAD line 48 `Vec<State>`) | built | Keeps. E3050 and E3040 are rewritten by area 1 (numbers are handed over too, verdict 84), but the lattice is generic (`SparseState`: `ABSENT`, `join_state`), so a new state only implements the trait. | `definite_init.rs` (area 1). | before M1 |
| K4 | Move-path states packed two to a byte: `States(Vec<u8>)` in `ember_analysis/src/drops.rs` (HEAD `Owned(u8)` line 44, `block_entry` line 245) | partly | `Owned` uses 3 of the 4 bits of a nibble. If `owns` for every type, `weakref` opening or `shares` hand-over need more than one more bit, the packing widens (to a byte per path). Check when area 1 and area 3 settle their states. | `drops.rs`: `check_borrowed_moves` (HEAD 984) and `check_scope_moves_all` (HEAD 1771) are rewritten by areas 1 and 2. | before M1 |
| K5 | Lists held at their counts: parser `body.rs`/`decls.rs`/`lib.rs`, `ember_ast`; the safety table written into one buffer (`safety_json` + `Entries` in `ember_codegen_c/src/lib.rs`); interface cache takes fresh artifacts (`prepare_interface_cache` now takes `Vec`, HEAD line 488) | built | Keeps. | `safety_json` entries carry the labels that [COST-5] says are renamed (RefCell borrow, dynamic exclusivity check); one small textual conflict in `codegen_c/src/lib.rs`. `parser/body.rs` also changed on main by ODR-280 (merged cleanly in the trial). | before M1 |
| K6 | C unit text goes to a `UnitSink` as each module's unit is made (`emit`, `emit_with`, `ember_driver` incremental) | built | Keeps. | The 14-argument `emit_with` signature (see section 2). | before M1 |
| K7 | Counting allocator behind the `count-allocations` feature, `--timings` shows each stage's live and peak MB: `ember_driver/src/memory.rs` (new, 79 lines), `Cargo.toml` | built | Keeps; it is how the rework's own memory (B9) is measured. | none (new file, one `mod memory;` line in `ember_driver/src/lib.rs`). | before M1 |
| K8 | Curve tooling and gates: `tools/memory_curve.py` (new, 235 lines), `tools/compile_budgets.py` (B9-curve; B9's 3 GB removed; B9 gated before/after as [BUD-3] gates times), `bench/bigpkg/generate.py --fanout`, `.github/workflows/release.yml`, `milestones.rs` (expects `B9-curve`, one bound only) | partly | Keeps, but **the curve is not flat**, so B9-curve fails today: the release workflow reports it only (`|| echo ::warning`), and `compile_budgets.py` exits 1 with an unflat curve, which breaks the laptop gate of record. Decision needed on the gate (shared decision S1). BUD-3 itself (a figure worse than before fails) is unchanged in 0.9.15 and B9 before/after is its memory form. | `milestones.rs` (the runner also gets `count_bits` in M0). | before M1 |
| K9 | **Not done:** the flat curve. The type check (one `Checker` over the whole program), the MIR passes and C emission each hold the whole program at once (AK notes, Q1) | not built | Cannot be done as AK planned it ("module by module from summaries") because of the new effect rules: EFF-1 and EFF-2 make a function's summary depend on the callees and, through a function value, an interface, a `dyn` or a `virtual` method, on "every function the program can put there", libraries included. That set is only complete for the whole program. What can stay flat is the *size* of what is held: summaries (small, per function, in the interface records) instead of bodies. So the work is to design the stages around the summary, after the effect summary exists (area 4). | `ember_typeck` (62k lines), `ember_mir`, `ember_codegen_c`: everything. | last (A9-12) |

### 3.2 AJ: a type of its own for every function and lambda (ODR-278)

Rules named by the helper: CLO-1, FN-6, CLO-3, CLO-10, EFF-1, EFF-2, TYP-5, TYP-23, MNG-1, BLD-1, COST-3.
§H.41: all of them are `changed` (CLO-13 is `removed`; AJ also touches it; LT-22 and LT-23 `changed`).
ODR-278 is an owner decision ("every function and every lambda has a type of its own", the owner's `fast`
and `make_list` example, D-901). The 0.9.15 text states the unique type for lambdas and local functions
([CLO-1]) and the per-function instance for an explicit type parameter ([CLO-14]); it does not state it for
`f = add_one`. See question Q1.

| # | what the branch changes (crate/file) | status | under 0.9.15 | conflicts with the rework's likely edits | merge |
|---|---|---|---|---|---|
| J1 | `TyKind::FnItem { symbol, name, shape }`: size 0, `Copy`, no drop, `Send`+`Sync`, `item_shape`, `join_items`, display `fn(A) -> R {name}`; `KindRecord::FnItem` in `codec.rs` (`ember_types/src/lib.rs`, +88; HEAD `TyKind::Fn` has the `stateless` flag at line 223) | partly | `shape` is a stateless `fn(A) -> R`. New [CLO-3] has one callable type wherever it is written, so the stateless / owned-callable split (ODR-102) disappears and `shape` becomes that one type; `thread_shared fn(...)` replaces `+ Send` ([FN-6]); the item is `ThreadShared` ([THR-9]). Zero size, `Copy` and "named function / capture-free lambda has no state and allocates nothing" agree with [CLO-10]. | `ember_types/src/lib.rs` (new type kinds for `shares`, `weakref`; `TyKind::Fn` flags change), `codec.rs` (schema). | before M1 |
| J2 | Item made where a function becomes a value: `fn_value_of` (HEAD 49618), `synth_lambda_fresh` (HEAD 48173), `item_symbol`, `free_function_symbol`, `item_defs` debug check (`ember_typeck/src/lib.rs`, +296 -53) | partly | Lambdas and local functions: [CLO-1]. Named functions kept by a local: needs Q1. Symbols stay valid ([MNG-1] injective, symbol is the identity), and a kept body's item is found again by symbol ([BLD-1]). | `synth_lambda_fresh` is rewritten by area 4 (capture words after the brackets, [CLO-2]; no `owned fn`). | before M1 |
| J3 | Conversion item to shape at coercion sites: `item_to_shape` in `coerce` (HEAD 35322), `Option[fn]`, export tables, `==` beside a callable value | built | [TYP-5] rule 10 now reads "a function or lambda to a callable type it satisfies ([CLO-3])" and covers it. These conversion sites are exactly the places a function is "stored into" a callable type, which the new EFF-2 sums over (shared decision S2). | `coerce` is edited by almost every typeck milestone. | before M1 |
| J4 | Joins where two items meet: conditional, `match` arms, list and map literals, an open `[]` fixed by a push, a local assigned again (`reassigned`, built from the scan that [CLO-13] used, now also seeing destructuring targets) | partly | The spec is silent on a local assigned two functions (AJ Q1), on `match` arms (Q3) and on a one-element list (Q2). A collection of callable type holding the shape is [CLO-3] as written ("a collection's element is a function value of that type"), so Q2 is answered. Q1 and Q3 go to Q1 below. [CLO-13] is removed, so the scan must be kept under its own name and no longer serves a borrow rule. | `ember_typeck/src/lib.rs` where [CLO-13]'s code is deleted by area 4. | before M1 |
| J5 | Call through an item is a direct call: `synth_indirect_call` (HEAD 47699) takes the shape's signature; MIR rewrites to `FuncRef::Direct` (`ember_mir/src/lower.rs`, +44; item value is `Const::Void`, conversion `Const::Fn(symbol)`, HEAD `Const::Fn` at `ember_mir/src/lib.rs:1119`) | built | [CLO-1]: "a call through it is direct"; [CLO-14]; [COST-3]. | `ember_mir/src/lower.rs`: `Const::Fn` becomes a two-word `{code, state}` value ([CLO-10]); the conversion must produce `{code, 0}`. | before M1 |
| J6 | Effects: no change in `ember_analysis/src/effects.rs`; calls through an item are direct, so exact; D-877's `plain_function_values` (HEAD line 904) now sees only converted items; `cycles.rs` reads an item as owning nothing (+3 lines) | partly | New [EFF-2] / [CLO-17] sum over "every lambda and named function stored into that type anywhere in the program", with the type including parameter words, result, `thread_shared`, `unsafe`. D-877 keys on (argument count, result type) and returns `Effects`; the rework replaces it by summaries keyed by the callable type (S2). AJ makes the source of that index exact: only conversions put a function into a type. `cycles.rs` (HEAD `L3001` at line 589): an item owns nothing, so it cannot close a loop of owners ([E3032] counts "a function value's state"; an item has none). | `effects.rs` (areas 4 and 5), `cycles.rs` (area 3, E3032). | before M1 |
| J7 | Backends: C backend `void`, one byte as a member (`ember_codegen_c/src/lib.rs`, +20 -7); LLVM `Repr::Zst` (`types.rs`, `ffi.rs`); evaluator void (`ember_interp/src/lib.rs`); FFI callback trampoline calls an item's function with no state | partly | Keeps. **Defect 3 stays open and matters more:** the analysis decides from `TypeTable::layout` whether a function value's state fits inline, and the C compiler decides from `sizeof`. [CLO-10] now says a state of one word or less stays inline and a larger one is a block that "carries `Alloc`", so a one-byte zero-sized member in C that pushes the state over the inline word hides an `Alloc` the analysis then misses (`@noalloc` unsound). AJ recommended option (a): the C backend leaves zero-sized members out. Do it (A9-6). | `codegen_c/src/lib.rs` (every area edits C emission). | before M1 |
| J8 | A closure naming an enclosing variable that holds an item captures nothing for it (`CaptureWatch::items`, a local set to the function where the body begins) | remove | [CLO-2]: a name used from the maker is "the real variable, that very variable and not a copy, unless a capture word names it", and [CLO-4]: a function that uses real variables may not outlive them. AJ's rule accepts programs those texts seem to refuse. Q2. | `ember_typeck` capture analysis (area 4). | drop at A9-5 |
| J9 | `[CLO-13]`'s "assigned after" scan sees destructuring targets (defect 2) and its test | remove | [CLO-13] is removed in 0.9.15 (captures are real variables, not copies). The test `CLO-13/reject_a_variable_assigned_by_destructuring_later_keeps_the_borrow_rule.em` retires. The scan itself survives only as J4's `reassigned`. | same code as J4. | drop at A9-5 |
| J10 | A function reached through its module is a value (`lib.offset`, `E1010` before; private is `E1052`) | built | Independent bug fix; [FN-6] "A function is a value". | module-path lookup in typeck. | before M1 |
| J11 | New use of `E2020` ("each function has a type of its own", help names the shape) | built | `E2020` is live ("mismatched types", p17a-codes). Wording and the error page (area 7) follow Q1. | `docs/errors/E2020.md`. | before M1 |
| J12 | LT-22 / LT-23 / `tests/ui/borrow/B14` / `docs/errors/E3065.md` edited so the opaque value comes from `bundle if ... else crossed` | remove | `E3065` and `L3014` are retired in 0.9.15 (p17a-codes lines 292 and 307): "a result points into every view argument it may come from, or into what `from` names; nothing is left to infer". The three tests flip or retire and the page is deleted by area 7; take the deletion at merge-conflict time. | `docs/errors/E3065.md` (+10 -3 on the branch). | drop at A9-5 |
| J13 | Not done: `callable_values` kept (instances keyed by the function given for a callable parameter, HEAD `ember_typeck/src/lib.rs:2463`, `39356-39374`) | remove | Under the new [CLO-3] a plain `fn(A) -> R` parameter is a function value, not an implicit generic, so instances per function come only from explicit `[F: ...]` ([CLO-14]), where the type argument already is the item. `callable_values` is dead. | `ember_typeck` instances (area 4). | delete at A9-5 |
| J14 | Not done: foreign declaration's value keeps the C pointer type (AJ Q5) | built | [FN-6]: `extern "C" fn(...)` is a C function pointer; no item needed. | none | n/a |

### 3.3 AI: the rest of the LLVM backend (phase 9)

Rules named by the helper: CG-LL-1, CG-LL-2, CG-LL-3, HR-*, FFI-8, FFI-9, FFI-21, FFI-28, FFI-31, FFI-33,
ABI-1, ABI-3, BLD-FFI-5a, BLD-6, UNS-10, BRW-8, EXC-18, EXC-19, BEN-5, TST-28. §H.41: CG-LL-3 `changed`
(now names `noalias` for every `ref mut` parameter and never `this`, plus `readonly`, `dereferenceable(N)`,
`nonnull`); EXC-18 `removed`; EXC-19 `changed` (fields are separate places; no access words); HEAP-5
`removed`; CELL-5 to CELL-12 `removed`; UNS-10, UNS-10a, BRW-8, RC-5, EXC-2, TYP-22, CLO-6, STD-9, FFI-8,
FFI-21, FFI-33, BLD-FFI-5a, HR-3b, HR-8, HR-11, HR-12, HR-15 `changed`. FFI-28, FFI-31, FFI-9, ABI-1, ABI-3,
BLD-6, BEN-5, CG-LL-1, CG-LL-2 are not in §H.41.

| # | what the branch changes (crate/file) | status | under 0.9.15 | conflicts with the rework's likely edits | merge |
|---|---|---|---|---|---|
| I1 | Reload "planner and writer" split: what an image says of itself is decided once in `ember_codegen_c/src/image.rs` (new, 2110 lines, `Planner`/`Writer`); `reload.rs` (HEAD 1780 lines; branch -1501 +397) and `frames.rs` (HEAD 302; -281) keep only C text; the C output is byte for byte the same (1957 programs) | built | Keeps. It is where the changed reload rules land once for both backends: [HR-11] (a field's type schema includes `owns`/`shares`/`shares mut`/`thread_shared`/`weakref`/`ref` words), [HR-8] (manifest lists the facts of [HR-44]), [HR-3b] (frame records also list "the functions the rest of the function may still call"), [HR-15] (owners, links, views to rewrite). `image::is_compiler_struct` (HEAD `reload.rs:1735`) lists compiler structs by old names. | **Area 6 edits `reload.rs`/`frames.rs`; AI deletes or moves almost all of both.** Merging AI after area 6 has started editing them is the worst conflict in this job. | before M1 (hard) |
| I2 | LLVM backend hot-reload images: `ember_codegen_llvm/src/reload.rs` (new, 904 lines), `func.rs`, `glue.rs` (`object_layout`, HEAD 93), `dispatch.rs`, `closures.rs`, `runtime.rs`, `lib.rs`, `types.rs`; 40-byte object header (`Types::header`, `TypeTable::class_field_offsets_after(id, header)`; HEAD `class_field_offsets` at `ember_types/src/lib.rs:2434` hard-codes 24); `musttail` thunks; `-fms-runtime-lib=dll` on Windows | partly | Keeps the design. Constants change: [OBJ-1] header is **8 bytes (12 with a weak count)**, [HR-12] adds 16 (24 or 28), and there are no access words. The runtime ABI and the `EMBER_RL_*` header constants are compared by a unit test (`hr_8_reload_records_are_the_headers`), so that test catches a missed constant. A function value's `captures` table follows the new two-word value ([CLO-10]). | `glue.rs`, `types.rs`, `ember_rt.h` and `ember_types/src/lib.rs` (area 3's count header). | before M1; adapt at the header milestone (A9-8) |
| I3 | `ember_build`: `link_image` takes the image's units, `ir_flags`, `unit_object`, `link_cdylib` takes units (`lib.rs` +108, `reload.rs`, `cdylib.rs`) | built | Keeps. | none expected. | before M1 |
| I4 | Libraries under LLVM: `cdylib` and `staticlib` from IR plus a C shim; module protocol in IR (`ember_module_init` checks the ABI first, copies the host config by its size); header from `ember_codegen_c::library_interface`; `@sync` class exports in IR (`ember_codegen_llvm/src/ffi.rs`, `emit_class_exports`); shims named under the package; D-915 `header.rs`; the library tail is one driver function for both backends | partly | Keeps the machinery ([FFI-28], [FFI-31], [ABI-1], [ABI-3] unchanged). `@sync` is removed: [BLD-FFI-5a] now presents a `thread_shared` class as a copyable handle whose copies are atomic owners ([RT-8], [RT-7] limit), so `ClassExport`/`MethodExport` in `class_exports.rs` (branch +100 -8) and the IR twin change. AI's spec question is still open in 0.9.15 (Q3). | `class_exports.rs`, `header.rs`, `ember_codegen_llvm/src/ffi.rs` (area 6). | before M1; adapt in area 6 |
| I5 | D-911, D-913 and EXC-18/EXC-19 work: a store into a class field holding a `Box[dyn I]` takes the field's access word; `behind_dyn` in `ember_analysis/src/borrows.rs`; C `access_call`; LLVM `access`/`has_field_word`; evaluator `object.rs access` | remove | EXC-18 is removed, EXC-19 now says fields are separate places and objects carry no access words ([OBJ-1]: "no access words and no flags"), and `Box` is gone. The whole fix, and its two tests, go with the run-time exclusivity checks. | `borrows.rs` (area 2/4 delete `insert_shared_accesses_all`, HEAD 327), backends. | delete in area 4 (A9-5) |
| I6 | D-912 virtual slots in a `BTreeMap` (`borrows.rs`, HEAD `VirtualSlots` line 363) | remove | The pass it sits in (the access-word insertion) is deleted. The *principle* (iterate ordered maps so C and IR are reproducible, [BLD-1]) applies to the new "every implementation and override" sets of [EFF-2]. | `borrows.rs`. | delete with I5; keep the habit |
| I7 | Independent fixes: D-914 `VaList` is a pointer on both x86-64 ABIs (`types.rs`); D-916 a vectorcall foreign function keeps its convention through its counted wrapper (`FfiCounted.abi` in `ember_hir`, both backends); `byval` reload thunk is `tail` not `musttail`; `extern "vectorcall"` (`abi.rs`, +297); AAPCS64 classification (not run on AArch64) | built | Keep. [FFI-9] unchanged; [FFI-8] changed only in wording around ownership. | none expected (`abi.rs` is new ground). | before M1 |
| I8a | ODR-275 for `Mutex`, `RwLock`, `Atomic`, `Once`: their state sits in `UnsafeCell` (`std/src/sync.em`, +11 -5); MIR `state_ty`/`state_place`; `TypeTable::unsafe_cell_inner`, `cell_held`; `holds_a_cell` is the one fact for [BRW-8] and `readonly` (HEAD `ember_types/src/lib.rs:1710`) | built | [UNS-10] (changed): "`UnsafeCell` is the primitive beneath `Cell`, `Mutex`, `RwLock`, `Atomic` and `Once`"; [BRW-8] still names `Cell` or `UnsafeCell` as what keeps a `Copy` parameter out of registers. `Cell` stays recognised by name (it is `Copy` when `T` is, [CELL-4], and an `UnsafeCell` never is). | `std/src/sync.em` (area 8), `lower.rs` `lower_atomic` (HEAD 2167). | before M1 |
| I8b | ODR-275 for `RefCell`: its four fields in `UnsafeCell`s (`refcell_of`, HEAD `ember_typeck/src/lib.rs:19923`), `lower_refcell_borrow` (HEAD `lower.rs:2188`), `guarded_parent` in `check_refcell_call`/`is_guard_loan` (HEAD `borrows.rs` 4006 / 5371), `state_member` in the C backend, `refcell_for_guard` (HEAD `glue.rs:960`), the evaluator's `RefCell_` lookups | remove | `RefCell`, `Ref`, `RefMut`, `borrow`, `borrow_mut` are not Ember ([CELL-5]..[CELL-12] removed, `L3011` retired). Deleted whole by area 3's RefCell removal; the part to keep is I8a. | those same functions | delete in area 3 (A9-5) |
| I9 | ODR-276: every reference parameter of an Ember function is `nonnull dereferenceable(size of referent)`, and a shared one to a referent that holds no cell is also `readonly`; none for zero-sized referents, exports, C-callable function values or callbacks (`abi::Ext::Ref`, `body_signature` in `ember_codegen_llvm/src/func.rs`, HEAD line 78) | partly | [CG-LL-3] (changed) asks for exactly these, plus `noalias` on every `ref mut` parameter and never on `this`. AI's argument that no callee can end the referent rested on run-time retained copies and panics ([RC-5] old text, ODR-065). The new argument is compile-time: [RC-5] point 1 (no line or call in a view's life may end an object of that type, `E3060`), [BRW-22] (`noalias` derived, never assumed), and for a `thread_shared` object reached through a `weakref` link an atomic count held for the view's life ([RC-5] point 2). The attributes stay sound if those checks are complete; the six evidence tests change meaning (section 4). `Ext::Ref` is the single place where `noalias` for `ref mut` belongs. | `func.rs` `body_signature` (area 2 adds `noalias`); the test `cg_ll_3_references_are_dereferenceable_and_shared_ones_without_a_cell_readonly` in `ember_driver/tests/llvm.rs` builds `RefCell` programs. | before M1; adapt in area 2 (A9-10) |
| I10 | ODR-274 profile-guided optimisation (`--pgo-generate`, `--pgo-use`, `[profiles.*] pgo`, `llvm-profdata` merge, the help line naming `libclang-rt-21-dev`) | built | [BLD-6] unchanged and not in §H.41. | `ember_driver/src/{lib,llvm}.rs` (CLI options). | before M1 |
| I11 | ODR-277 measured and not built; `tests/perf/int_math.em` rewritten to add once a pass as its C twin | built | Keep (speed baseline; a gate program). | none | before M1 |
| I12 | Driver tests that embed Ember sources: `ember_driver/tests/{llvm,reload,cdylib,staticlib}.rs` (+237, +131, +73, +208); `milestones.rs` | partly | They carry the LLVM parity matrix (reload 50, cdylib 11, staticlib 6, llvm 12). Their inline programs use old spellings (`RefCell`, `Box`, `Shared`, `@sync`) and must be respelled per milestone. | `reload.rs` tests also change on AJ's branch (`removing_a_function_whose_address_is_held_is_refused`). | before M1 |

## 4. The branches' own tests against 0.9.15

`keep` = valid as written; `respell` = same meaning, spellings change; `rewrite` = expectation changes;
`retire` = tests something removed. I read every file listed below by hand (the old-spelling counts came from
a grep; the verdicts are mine).

### AJ (18 entries: 16 tests added or changed, 1 support file, 1 test deleted)

| class | files |
|---|---|
| keep (10) | `CLO-1/accept_a_lambda_in_a_generic_adapter_is_called_directly` (the `assert-c` names `ember_fn_value`; re-check the name when [CLO-10] changes the struct); `EFF-1/accept_a_call_through_a_variable_holding_one_function_has_its_effects` (needs Q1 = yes); `EFF-1/accept_an_adapter_holding_a_lambda_has_its_effects`; `EFF-1/reject_a_variable_holding_either_function_has_both_effects`; `FN-6/accept_a_function_has_a_type_of_its_own_that_converts_to_its_shape` (joins need Q1); `FN-6/accept_a_function_reached_through_its_module_is_a_value`; `FN-6/reject_a_private_function_of_another_module_is_no_value`; `FN-6/items/steps.em` (support); `LT-22/accept_a_function_given_by_name_has_its_own_provenance`; `tests/run-pass/views_passed_as_two_words` |
| respell (1) | `HR-5/accept_a_call_goes_through_its_thunk` (one `virtual fn area(self)` line: `this`; plus the value representation) |
| rewrite (5) | `CLO-3/accept_a_function_argument_is_called_directly` (asserts one copy of `apply` per function for a plain `f: fn(int) -> int` parameter: under the new [CLO-3] that is a function value; write `[F: fn(int) -> int]` to keep the claim); `CLO-1/reject_a_place_holding_one_function_holds_no_other` (E2020 and AJ Q6 depend on Q1); `LT-22/reject_opaque_multi_region_result_provenance`, `LT-23/reject_borrows_cannot_stand_in_for_unknown_field_provenance`, `tests/ui/borrow/B14/multi_region_result_provenance` (all expect `E3065`, retired; the result now points into every view argument it may come from, so the program is accepted or refused for a different reason) |
| retire (2) | `CLO-13/reject_a_variable_assigned_by_destructuring_later_keeps_the_borrow_rule` ([CLO-13] removed); `EFF-1/reject_a_plain_fn_value_of_a_shape_some_function_allocates_in` (AJ deleted it; the branch replaced it by the adapter test, so the merge must take the deletion) |

Rust tests of AJ: `ember_types` unit test `a_function_item_holds_nothing_and_meets_another_at_its_shape` (keep);
`ember_driver/tests/incremental.rs` `function_items_are_kept_across_builds_as_a_clean_build_makes_them` (keep;
schema constants will change); `llvm.rs` `clo_1_a_lambda_in_an_adapter_is_called_directly` (keep);
`reload.rs` change (respell with AI's); `milestones.rs` (+4: `word_len` splits its view on Windows; keep).

### AI (18 `.em` files, 4 support C/H files, plus the Rust tests)

| class | files |
|---|---|
| keep (3) | `FFI-8/accept_a_newtype_in_a_struct_crosses_by_value` and `FFI-8/overlays/stamps.em` (with `stamps.c`, `stamps.h`); `tests/perf/int_math.em` |
| respell (2) | `FFI-21/accept_a_nullable_retained_callback_keeps_an_argument_and_the_result` (`owned token` is `owns token`; support `channels.c`, `channels.h` keep); `STD-9/accept_a_held_print_formats_aggregates_and_runs_code` (`fn fmt(self, mut f)` to `this`) |
| rewrite (10) | `CLO-6/accept_a_boxed_callable_moved_into_its_call_ends_once` (`Box[fn]` is gone, `owned` to `owns`, `self` to `this`; keep the `once fn` moved-into-its-call claim); `EXC-19/accept_a_dyn_field_is_stored_through_its_word` (no access word; keep the program as an `owns dyn Speak` field-store run test); `EXC-2/run_fail_a_callee_clearing_the_list_an_element_borrow_came_from` and `..._replacing_the_list_...` (run-time panic becomes a compile-fail: the call may change what the view reads, `E3021`); `RC-5/accept_a_callee_ending_the_owner_a_borrow_points_into`, `RC-5/accept_a_callee_replacing_the_field_a_borrow_went_through`, `RC-5/accept_a_callee_dropping_one_owner_of_a_borrowed_shared` (compile-time [RC-5] point 1, `E3060`, `Shared` to `shares`); `TYP-22/accept_a_box_dyn_lends_its_value_as_a_ref_dyn`, `TYP-22/accept_a_box_dyn_of_an_interface_requiring_clone_and_eq` (`Box[dyn I]` is `owns dyn I`; [TYP-22] changed); `UNS-10/accept_what_a_shared_reference_changes_is_in_a_cell` (drop the `RefCell` case, add a `shares mut` case; keep `Cell`, `Atomic`, `Mutex`) |
| retire (3) | `CELL-9/accept_the_counter_is_one_machine_word` ([CELL-9] removed); `EXC-18/run_fail_a_view_through_a_dyn_field_holds_the_object` ([EXC-18] removed); `HEAP-5/run_fail_a_callee_replacing_the_shared_a_borrow_points_into` ([HEAP-5] removed) |

Rust tests of AI that need respelling or rewriting per milestone: `ember_driver/tests/llvm.rs`
(`cg_ll_3_references_are_dereferenceable_and_shared_ones_without_a_cell_readonly` builds `RefCell`, `Cell`,
`Mutex`, `Atomic` programs), `reload.rs` (50), `cdylib.rs` (11), `staticlib.rs` (6; the `@sync` class export
test `a_sync_class_crosses_a_static_librarys_exports`), and the `hr_8_reload_records_are_the_headers` unit test
(it compares the header constants and will fail loudly until they change).

### AK (no conformance tests)

Rust tests: `ember_analysis` unit tests `sparse_states_are_the_dense_ones`, `packed_states_are_the_states_they_hold`
(keep; they are brute-force equivalence tests); `ember_driver/tests/milestones.rs`
`the_compilers_speed_is_measured_before_and_after_a_change_and_published` (keep; needs the gate decision S1).

Totals over the 36 entries above (AJ 18, AI 18): keep 13, respell 3, rewrite 15, retire 5.

## 5. Work items

Ids are `A9-n`. Part A are the merge steps. Part B are the changes each merge hands to a rework milestone;
I list them here so the other areas' reports can name them. Sizes: S under a day, M days, L a week or more.
"Design tests" are named only where the rule map (`conformance-rule-map.md`, spec-ID column) names a rule
these items complete; 347 of the 841 are mapped only by main ID, so these counts are lower bounds.

### Part A: merge steps (on `main`, before M1, at the old spec)

**A9-1. Merge AK.** S. Order: first.
- Files: the 25 listed in section 2; no textual conflict expected.
- Check: the full check; the C and diagnostics of the 3,435-program set byte-identical to the compiler
  before the merge (AK's own proof); B9 before/after; `-j 1` against `-j N`; incremental against clean.
- Rules: BLD-1, BLD-2, BLD-7, BUD-1, BUD-2, BUD-3 (as they stand; none is in §H.41 except BLD-2).
- Tests: none new. Old tests touched: `milestones.rs` budget test.
- Depends on: nothing. The gate decision S1 (below) must be taken before the push.

**A9-2. Merge AJ.** M. Order: second, after AK's full check.
- Files: the 15 compiler/docs files in section 3.2 and 18 test files; the trial merge with HEAD and with AK is clean.
- Check: AJ's 18 test files (C, LLVM, evaluator agreeing), `ember_types` unit test, `incremental.rs`,
  `llvm.rs`, `reload.rs`, the five `tests/perf` map programs' C (AJ says 70 of 75 programs emit identical C),
  then the full check. Take the deletion of `EFF-1/reject_a_plain_fn_value_of_a_shape_some_function_allocates_in.em`.
- Rules (old spec): CLO-1, CLO-3, EFF-1, FN-6, TYP-5, TYP-23, MNG-1, COST-3 as they stood.
- Depends on: A9-1. Open: Q1 and Q2 are answered after the merge, not before it (the old spec allows both).

**A9-3. Merge AI.** M (the work is the verification, not the text). Order: third.
- Files: 65; resolve the 14 hunks named in section 1, with AE's resolutions in `3f2fce5b` and AI's own
  "Merge notes for the lead" (its `Planner` takes AE's `reload.rs` decisions; AE's `frames_on` change applies to
  what is left of `frames.rs`; `ember_driver::reload::build` takes a list of units).
  `emit`/`emit_with` in `ember_codegen_c/src/lib.rs` gets both `unit_sink` (AK) and AI's changes in one pass.
- Check: C backend with the evaluator, all tests; `EMBER_BACKEND=llvm EMBER_PROFILES=all` (AI reports 2144 of
  2144); reload 50, cdylib 11, staticlib 6 under both backends; llvm 12; the C warning gate over the folders the
  C changes touch; on Windows and in WSL (clang 21, gcc). `sudo apt install libclang-rt-21-dev` is needed for the
  PGO test in WSL (the owner must run it; AI's notes give the command).
- Rules (old spec): CG-LL-1..3, HR-*, FFI-8, FFI-9, FFI-21, FFI-28, FFI-31, FFI-33, ABI-1, ABI-3, BLD-FFI-5a,
  UNS-10, BEN-5.
- Depends on: A9-2. Hard requirement: before area 6 touches `reload.rs`/`frames.rs`, and before the first
  milestone that changes the object header.

**A9-4. Record the gates the rework runs under.** S. Order: with A9-3.
- Files: `tools/compile_budgets.py`, `.github/workflows/release.yml`, the full-check script `scratchpad/verify_nobench.sh`.
- Content: (a) B9-curve is reported, not gating, until A9-12 makes it flat (see S1); (b) the LLVM matrix
  (`EMBER_BACKEND=llvm EMBER_PROFILES=all`, reload, cdylib, staticlib) joins every milestone's full check, as
  [CG-LL-1] wants both backends to pass the conformance and performance suites; (c) the baseline list of
  checks that are red on this branch because they are tied to the new spec (M0 turns them green).
- Depends on: A9-3.

### Part B: what each merge hands to a milestone

**A9-5. Delete the dead parts, in the milestone that removes their rule.** M in total, three pieces.
- (a) area 3, with RefCell removal: delete I8b (`refcell_of`, `lower_refcell_borrow`, `check_refcell_call`,
  `is_guard_loan`'s RefCell branch, `guarded_parent`, `state_member`, `refcell_for_guard`, the evaluator's
  `RefCell_` lookups); keep I8a. Rules: CELL-5..CELL-12 (removed), CELL-13, UNS-10.
- (b) area 4, with the run-time exclusivity checks: delete I5 and I6 (`behind_dyn`, `access_call`, `has_field_word`,
  `object.rs access`, the `VirtualSlots` pass `insert_shared_accesses_all`); retire `EXC-18/run_fail_...`,
  `EXC-19/accept_a_dyn_field_is_stored_through_its_word` (rewrite), `HEAP-5/run_fail_...`. Rules: EXC-18, EXC-19, HEAP-5.
- (c) area 4 and 7, with the function-value milestone: AJ's J8, J9, J12, J13 (items-capture-nothing, the
  [CLO-13] scan and test, the `E3065` tests and page, `callable_values`). Rules: CLO-2, CLO-4, CLO-13, LT-22, LT-23.
- Depends on: A9-2, A9-3; the milestones that remove those rules (areas 3, 4, 7).

**A9-6. C backend leaves zero-sized members out of its structs (AJ defect 3, option a).** M.
- Files: `ember_codegen_c/src/lib.rs` (struct emission, field reads, writes, initialisers, the callback record),
  `ember_types/src/lib.rs` layout agreement test (`TypeTable::layout` against `sizeof` in the generated C).
- Rules: CLO-10 (state of one word or less is inline; a larger one is a block that carries `Alloc`), EFF-1 (`Alloc`
  effect from creating a function value with state), COST-3.
- Tests: add a conformance test that `@noalloc` is refused exactly when the C state exceeds the inline word,
  for a state that holds a function item. Old tests: none change.
- Depends on: A9-2; with the function-value representation (area 4: needs `{code, state}`).

**A9-7. Adapt AJ's item type to the one callable type and the two-word value.** M.
- Files: `ember_types/src/lib.rs` and `codec.rs` (`shape` is the one callable type; schema bump),
  `ember_typeck/src/lib.rs` (`item_to_shape`, `fn_value_of`, `coerce`), `ember_mir/src/lower.rs`
  (`Const::Fn` as `{code, 0}`), both backends and the evaluator.
- Rules: CLO-1, CLO-3, CLO-10, CLO-14, FN-6, TYP-5, TYP-23, MNG-1.
- Tests: `CLO-3/accept_a_function_argument_is_called_directly` rewritten with `[F: ...]`; the design tests that name
  CLO-1 (68: R12 23, R15 28, R3 4, R11 4, R4 2, R8 3, R14 1, R5 1, R17 2), CLO-3 (10), CLO-6 (7), CLO-4 (9).
- Depends on: area 4's two-word function value and capture-word work; Q1 and Q2.

**A9-8. Header sizes and layout rows in AI's reload code.** M.
- Files: `ember_types/src/lib.rs` (`class_field_offsets_after`, HEAD 2434 has the 24-byte constant),
  `ember_codegen_llvm/src/{glue,types,reload}.rs`, `ember_codegen_c/src/{image,reload,frames}.rs`,
  `runtime/ember_rt/include/ember_rt.h`, the runtime ABI version, and the unit test that compares the headers.
- Rules: OBJ-1 (8 or 12 bytes), HR-12 (24 or 28), HR-11 (words in a field's type schema), HR-8 ([HR-44]
  facts in the manifest), HR-3b (frame records list functions still to be called), HR-15 (owners, links and
  views rewritten on a move), HR-12a (`E9035` agreement at link time).
- Depends on: area 3 (the count header and the weak count, shared decision S5) and area 6.

**A9-9. Class exports for `thread_shared` handles in both backends.** M.
- Files: `ember_codegen_c/src/class_exports.rs`, `header.rs`, `ember_codegen_llvm/src/ffi.rs`
  (`emit_class_exports`), `ember_driver/tests/staticlib.rs`.
- Rules: BLD-FFI-5a, FFI-31b, FFI-28 (Q3), RT-7/RT-8 (limit-checked atomic count).
- Depends on: area 5 (`thread_shared` classes) and area 6; Q3.

**A9-10. Re-ground ODR-276 and add `noalias`.** M.
- Files: `ember_codegen_llvm/src/{abi,func}.rs` (`Ext::Ref` carries `noalias` for `ref mut`, never for `this`;
  `body_signature`), the C backend's `restrict` choice, `ember_driver/tests/llvm.rs`, and the six evidence tests
  (section 4: RC-5 ×3, EXC-2 ×2, HEAP-5 retire).
- Rules: CG-LL-3, BRW-22, BRW-8, RC-5 (points 1 and 2), SIMD-3 (derived, never assumed), UNS-10.
- Tests: the six rewritten compile-fail tests (RC-5, 5 design tests: R10 2, R11 3); a case that proves `readonly` is
  absent for `Cell`, `Atomic`, `Mutex`, `RwLock`, `Once` parameters and present for plain shared referents.
- Depends on: area 2 (the checks that make the attributes sound, BRW-1/11/22, RC-5 point 1).

**A9-11. Build the "stored into a callable type" index from FnItem conversions.** S to M, inside area 4's
effect summary (see S2). Files: `ember_analysis/src/effects.rs` (replaces `plain_function_values`, HEAD 904),
the MIR `Cast`/`Const::Fn` conversion sites. Rules: EFF-1, EFF-2, EFF-24, CLO-17, EFF-26. Depends on A9-2 and
area 4's summary representation. Tests: `EFF-1/accept_a_call_through_a_variable_holding_one_function_has_its_effects`
and its two siblings must keep passing with a callable parameter that is *not* a type parameter refused as
[EFF-2] says (every function stored into that type).

**A9-12. Finish ODR-281 on the reworked pipeline.** L. Last of this job; after the effects milestone.
- Files: `ember_typeck` (the checker holds one program), `ember_mir` passes, `ember_codegen_c` emission,
  `ember_build/src/interface.rs` (summaries in the records), `tools/memory_curve.py`.
- Rules: BUD-1, BUD-2, BUD-3, BLD-1, BLD-2 (changed), BLD-14.
- Done when: B9-curve is flat (peak at the largest size within 10% of the smallest, clean and after edits) with the
  summary-based stages; then B9-curve becomes a gate (S1).
- Depends on: area 4's summary; A9-1.

## 6. Shared decisions

**S1. Where and how the merges are gated.** Propose: merge on `main` at the old spec (A9-1..A9-3) before M1, then
bring this branch up to date with `main` (the compiler directories are identical, so that merge brings only the
helpers' changes). Reason: AK's and AI's proofs are byte-identity and parity claims that cannot be repeated once
M1 changes outputs; AI's reload split must precede area 6; the textual conflict count (0, 0, 14) only grows as
milestones edit `typeck`, `lower.rs`, `reload.rs`. Cost: delays M1 by the three full checks (the helper notes report
them done on the helpers' own trees, so the lead repeats them once). Gate policy: B9 before/after stays a gate
([BUD-3], unchanged); B9-curve is reported until A9-12 makes it flat. This is not a spec change: ODR-281's "passes
only when flat" is the owner's, and the curve is knowingly not flat.

**S2. How a function value's possible targets are found.** Propose: an index from the callable type (parameters with
words, result, `thread_shared`, `unsafe`, `once`) to the set of functions converted to it, built from AJ's
conversion sites (the `item_to_shape` `Cast` and `Const::Fn` operands in MIR), per module, kept in the interface
record, and merged over the program when the summary of a function with a call through that type is closed.
It replaces D-877's key (argument count, result type) in `effects.rs:904`. A call through an item type is a
direct call and needs no index. Cost against C: none at run time (compile time only); a call through a callable type
is one indirect call (as C's function pointer) with one extra state word; a call through an item is a direct call (C++
lambda, Rust closure). The index makes a module's key depend on a global table, so the key of a caller must include a
hash of the slice of the table it used, or BLD-2's "re-check callers only if a fact they use changed" degrades to
"any module that converts a function re-checks every caller through that type". That is area 4's to design; I flag it.

**S3. The function value and the item.** Propose: the item has size 0 (C backend: omitted from structs, S6), converts
to `{code, 0}`; a function value with captured state of one word or less keeps it in the state word, larger state is
one block (`Alloc`). This is [CLO-10] as written and removes AJ's three-word inline rule (old [CLO-10]).

**S4. One fact decides `readonly`.** Keep `TypeTable::holds_a_cell` (AI's ODR-275 made it complete) as the single
fact for [BRW-8] and `readonly`. A `shares mut` payload is not a cell: its changes are compile-time checked
([RC-5], [BRW-1], [BRW-11]) and the count lives in the header, which a `ref` parameter's pointer is never used to
write. Decide this explicitly in area 3 and add a test; if any path lets a callee write through a `ref` into a
`shares mut` payload while the caller reads it, `readonly` would be unsound.

**S5. Header size is one function.** AI parameterised the header (`class_field_offsets_after(id, header)`); the
rework should keep that shape and make the header size come from a single function of (reload build, weak-countable
family) = 8, 12, 24 or 28, used by the layout, both backends, the runtime header and the reload manifest. Owned by
area 3's count-header decision; AI's `hr_8_reload_records_are_the_headers` test then guards it.

**S6. Zero-sized members are omitted in C.** Cost against C: saves bytes; no run-time cost; makes the analysis's layout
and `sizeof` agree.

**S7. What AK needs from the effect summary.** Compact, per function, bounded by the function's callees and the
callable types it calls through (not by bodies), stored in the interface record with a schema at least 16; the
whole-program stage reads summaries and never bodies. Without this, A9-12 cannot make the curve flat.

## 7. Questions for the owner

**Q1. Does a named function have a type of its own in 0.9.15, and what does a local holding one do when assigned
another?** Rule texts: [CLO-1] "A lambda or local function has a unique type of its own ... Given to a type parameter
([CLO-14]) it keeps that type, and a call through it is direct; wherever a callable type is written ([CLO-3]) it
becomes a function value"; [CLO-14] "`F` names the exact function given"; [TYP-23] "A local declared by `x = e`
takes `e`'s type"; [EFF-2] "For a parameter, field, list element or local of a function type, it is every lambda
and named function stored into that type anywhere in the program". ODR-278 (yours, "yes", 2026-10-23, D-901) says
`action = add_shield` is of type `add_shield`, so that the `@noalloc fast` / allocating `make_list` example compiles;
that is not written in 0.9.15 for a named function kept in a local, and nothing says what `if c: f = a else: f = b`
does. Why it blocks: A9-7 either keeps or removes AJ's join rules (local assigned twice holds the callable type;
`match` arms; one-element list) and the tests `EFF-1/accept_a_call_through_a_variable_holding_one_function_has_its_effects`
and `FN-6/accept_a_function_has_a_type_of_its_own_...`; without a ruling the exact-effects example is refused again
([EFF-2] read literally sums every function stored into `fn(int) -> int`). AJ's proposed texts for [FN-6], [CLO-1],
[CLO-3], [TYP-5], [TYP-23], [EFF-2], [MNG-1], [COST-3], [CLO-10] are in `tasks/95/AJ-notes.md` (the lead writes the
spec text). Recommendation, if you want one: the local holds the callable type when assigned again (AJ's option a:
costs nothing, breaks no program).

**Q2. Is a variable that holds a function item a "real variable" for [CLO-2] and [CLO-4]?** Texts: [CLO-2] "A name a
lambda or a local function uses from the function that made it is the real variable, that very variable and not a
copy, unless a capture word after its brackets names it"; [CLO-4] "one that uses real variables may not outlive
them". AJ made a closure that names a variable holding an item capture nothing for it, so
`f = add_one`, then `return fn() => f(1)` compiles; read literally, the lambda uses the real variable `f` and
outlives it. Why it blocks: A9-5(c) must delete the rule or keep it; I propose deleting it (the text is clear and
a zero-sized variable is still a place), so say if the zero-sized case should be an exception. This is the only place
I found where an AJ behaviour accepts a program the new text appears to refuse.

**Q3. May a `cdylib` export the functions its class handles need?** Texts: [FFI-28] "`kind = "cdylib"` a shared
library exporting only its `@export` symbols, its `@export_table`s, `ember_module_init`, `ember_module_shutdown`, and
the marker"; [BLD-FFI-5a] (changed) "the header presents [a `thread_shared` class value] as a copyable handle class of
the same name with the class's `pub` methods, each copy one more owner on the value's atomic count and each end one
fewer". A `cdylib` host that includes that header cannot call the retain, release and methods it declares, because the
library does not export them (AI found this, both backends, and left it as built). Why it blocks: A9-9 builds the
handles for both backends and must either export them from a `cdylib` (AI's recommendation, they carry the package
name as prefix and cannot clash) or leave them out of a `cdylib`'s header.

## 8. Risks and what I could not verify

- **The milestone plan does not exist yet.** "Before M1" and "at the header milestone" name events, not dates; the
  lead's `MILESTONES.md` will fix them. If the rework starts on this branch before the merges, every conflict count
  above is a lower bound.
- **Semantic conflicts are not visible to a textual trial.** AI's 15 other both-changed files merged textually
  (`ember_typeck/src/lib.rs`, `ember_mir/src/lower.rs`, `ember_types/src/lib.rs`, `ember_analysis/src/borrows.rs`,
  `ember_build/src/lib.rs`, ...), but only a build shows whether AE's changes and AI's `Planner` still agree.
  I did not build anything. The trial used `git merge-tree --trivial-merge` against this branch's tip; AI's merge
  with AK and AJ applied was not tried (it would write objects).
- **AK's whole premise is at risk (K9).** If EFF-2's whole-program set can only be closed at the program, then
  "peak memory set by the biggest module" holds only if what the program stage reads is the summaries. I have not
  shown it can be flat, only that bodies need not be held. The size of the summaries (every function, plus the
  stored-into index) is unmeasured; it is linear in the program, so a flat curve may become a very shallow slope.
  Whether the owner accepts a slope smaller than today's 12 MB per 1,000 lines is a question for later, not now.
- **AK's nibble packing** has one spare bit (K4). A fourth state for a path (for example "opened link") makes the
  packing wrong; the unit test would catch it only for states it generates.
- **`dereferenceable` and `readonly`** now rest on compile-time checks only (I9, S4). Any hole in [RC-5] point 1,
  [BRW-11] or the effect summary becomes undefined behaviour in the LLVM backend instead of a run-time panic. The
  six evidence tests become compile-fail tests; they are the only tests I saw that show a callee ending a view's
  object, so they must be rewritten, not dropped.
- **AJ's measurements are not final**: its timed rounds were taken on a loaded machine; only the Cachegrind counts
  are exact (c1: -55.8%, the others unchanged). The notes' figures for AK are "TBD". Both want a rerun on the
  laptop with P-core affinity before the merge is called done.
- **AI's PGO test needs a package the owner installs in WSL** (`libclang-rt-21-dev`); by your rule I should name the
  exact command: `sudo apt install libclang-rt-21-dev`. AI's notes also say AArch64 was classified and not run.
- **A small defect in AK's diff**: `ember_build/src/interface.rs` has a doc comment turned into `////` (a plain
  comment) above `prepare_interface_cache`; harmless to the build, but the doc line is lost. Fix at merge time.
- **Old spec named in test expectations.** Every merge in Part A is judged against the old spec; some behaviour they
  add (the `E3065` tests, [CLO-13], `callable_values`, access words, RefCell) is correct there and is then removed on
  purpose by the milestone that retires the rule. That is intended, but the lead should not read those removals as
  regressions.
- **Confidence.** High for the sizes, the file lists and the textual conflict results; high that the rule changes
  I cite are as the spec says (I read the new and old texts); medium for which of AI's and AJ's code the rework will
  delete (it depends on how areas 3 and 4 slice their milestones); low for the effort of A9-3's verification and for
  A9-12.
