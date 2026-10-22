# Helper R — notes (2026-10-22)

Rules: DIA-15, CG-C-9, PHIL-1, CG-C-7, VER-3, CG-C-6, CG-C-10, DIA-5, LT-39, SEL-2, ALC-2, OPT-1.

## SEL-2 — done
- Tests: `tests/conformance/SEL-2/reject_shared_and_cpp_shared_never_convert.em` (argument, return,
  annotated local, `as` cast; `Shared` and `Weak` to `CppShared` and back),
  `tests/conformance/SEL-2/accept_shared_and_cpp_shared_each_keep_their_own_count.em`.
- Compiler: `compiler/ember_typeck/src/lib.rs` — `counted_family` / `counted_family_mismatch`;
  `coerce` and the `as` cast now report `E5065` (was a plain `E2020`) when one side is Ember's
  `Shared`/`SyncShared`/`Weak` and the other C++'s `std.ffi.CppShared`/`std.ffi.CppWeak`.
- `docs/errors/E5065.md` written (the code had no page; `ember test --doc` passes both examples).
- Gap outside SEL-2: `std.ffi` has no `CppWeak` (the `[FFI-17a]` table maps `std::weak_ptr<T>` to
  it). The check already names `std.ffi.CppWeak`, so it covers it once it exists.

## DIA-15 — done
- Tests: `tests/conformance/DIA-15/reject_suggestions_come_from_scopes_types_and_exports.em`
  (scope table: local; type information: field and method, only the receiver's own methods; index
  of exported names: a value and a type), `reject_an_import_line_is_of_the_right_kind.em` (a
  function is not offered where a type is expected; a type is where a value is),
  `accept_offered_import_line_compiles.em`.
- Compiler:
  - `compiler/ember_typeck/src/lib.rs` — `method_name_suggestions`: shape N3's "nearest member" was
    missing (`p.length_squard()` offered nothing); now ranked from the method table as `[DIA-24]`
    ranks names, receiver and its bases only, invisible methods left out.
  - Shape N2 ("name exported elsewhere: the import line, as a fix-it") was not built. New
    `ember_typeck::ExportIndex` trait (+ `Exported`), a new `exports` argument to
    `ember_typeck::check`, and `add_import_suggestions`: up to three `from M import name` fix-its,
    inserted after the module's last import (or at the top of the file), of the right kind for the
    position. Used for unknown values, unknown callees and unknown types.
  - `compiler/ember_driver/src/main.rs` — `Exports`, the index: every `pub` item of std's modules and
    every `pub`/`pub(package)` item of the root package (its tree when it has a manifest, else the
    files beside the root file), parsed (never type checked) lazily — only the first time a name is
    not found, so a program that compiles never builds it. Passed at all three `check` calls.
- Ceiling: dependency packages' modules are not in the index (their import paths are rewritten per
  package); add them when a dependency's names need offering.

## LT-39 — done (with a spec question)
- Tests: `tests/conformance/LT-39/reject_a_field_selected_at_run_time_needs_every_region.em`,
  `tests/conformance/LT-39/accept_regions_a_selection_cannot_reach_may_end.em`.
- No compiler change: a run-time choice between fields (a conditional expression, or a function
  choosing by an argument) already holds every region it could select, and only those, and they
  end at the selection's last use.
- The rule's examples, reflection and field descriptors, have no run-time operation that selects a
  field in safe Ember: `TypeInfo`'s `FieldDesc` (`[RFL-2]`) gives an offset, which reaches a field
  only through a raw pointer in `unsafe`, outside region tracking. So every operation the rule can
  apply to today is covered.
- Spec question: the rule says "`E3065` if a region it needs has ended", but `E3065` is defined
  (registry, `[LT-35]`, shape B14) as "multi-region result provenance cannot be inferred". Ending a
  region a live selection needs is an ordinary loan conflict, reported `E3021` (as every other
  ended-region use is, `[LT-18]`). Options: (a) change LT-39's parenthesis to name the loan
  conflict (`E3021`/`E3010` family, shape B1); (b) keep `E3065` and make the compiler report a
  run-time selection's ended region as B14. Recommendation: (a) — B14's help ("return an owned
  value, or the views as separate results") does not fit a local conditional, and the region is
  known, not uninferable.

## PHIL-1 — done
- Tests: `tests/conformance/PHIL-1/accept_identical_locals_share_lifetime_whatever_their_use.em`
  (identical locals used at different times both live to the block's end and drop in reverse
  order), `accept_identical_declarations_share_storage_and_synchronisation.em` (a struct passed
  `mut` and one only read are both frame values, no `ember_alloc`; a `@sync` object handed away
  and one kept local both release atomically, no plain release; checked on the emitted C).
- No compiler change: storage, lifetime and synchronisation already follow the declared type only.

## CG-C-6 — done (built)
- Tests: `tests/conformance/CG-C-6/accept_a_loop_in_vectorisable_form_gets_the_pragma.em` (an
  `@simd(assert)` loop: the pragma directly before its grouped `for`, once, release and debug),
  `accept_a_loop_not_in_vectorisable_form_gets_no_pragma.em` (an indirect index and an early
  `return`: none). Passed under MSVC and clang locally (no gcc on this laptop).
- Compiler: the pragma was not emitted anywhere.
  - `runtime/ember_rt/include/ember_rt.h` — `EMBER_VECTORISE`: clang `loop vectorize(enable)`, GCC
    `ivdep`, MSVC `loop(ivdep)` (their only loop-vectorisation pragmas; the assertion they make, no
    dependency through memory between iterations, is what `[SIMD-5]`'s disjointness and one-offset
    clauses prove), empty elsewhere.
  - `compiler/ember_analysis/src/loop_version.rs` — `loops_in_vectorisable_form`: the `[SIMD-5]`
    check (`vectorisable`) run on the final body for given loop headers; in form = every clause met
    and no check left (a grouped copy). Read from the body the backend writes, so copies and
    renumbering by earlier passes cannot misplace it.
  - `compiler/ember_codegen_c/src/lib.rs` — `emit_body` asks it for the counted loops it writes as
    `for`; `emit_for_loop` writes `EMBER_VECTORISE` before the `for` (and before the unrolled one).
- Speed: the owner's laptop bench should be run before pushing (GCC/MSVC now get `ivdep` on these
  loops; clang a vectorise hint).

## DIA-5 — done (built)
- Tests: `tests/conformance/DIA-5/` — `reject_an_error_names_the_header_and_the_c_declaration.em`
  (`E5002` at a call, `E5020` at an argument: each names `labels.h:N` and the declaration as written,
  a two-line declaration joined), `reject_an_overlay_error_names_the_c_declaration.em` (an error at
  the declaration an import wrote), `reject_a_cpp_base_error_names_the_class_declaration.em`
  (`E5058` on an Ember class deriving from a C++ class: `class Gauge {…}`),
  `accept_an_import_with_its_facts_stated_reports_nothing.em`; support `labels.h`, `gauges.hpp`,
  `overlays/`.
- Compiler: FFI errors named the header at most (in a help), never the C declaration.
  - `compiler/ember_driver/src/cimport.rs` — the importer records each top-level declaration of the
    header (functions, structs, unions, classes, enums, typedefs, aliases, variables) as the header
    writes it: its source extent on one line, bodies elided as `{…}`, no `;`, at most 160
    characters (`Writer::c_text`, `one_line`), with `file:line` (`Imported::declared`). Kept in the
    `.embind` record (`EMBIND_VERSION` 8 -> 9: older records are re-read from the header once).
    `ForeignImport::declarations` gives each module (and each C++ namespace module) its
    declarations by the names it binds them to (an overlay's rename too).
  - `compiler/ember_driver/src/main.rs` — an import's overlay modules carry its declarations.
  - `compiler/ember_typeck/src/lib.rs` — `c_declarations` (each such declaration's item span, or an
    overlay entry's), `note_c_declarations`: after checking, every FFI-subsystem error whose primary
    span or a secondary span lies in such a declaration, or that is at a call of one, gets the note
    "declared in the header at `file:line`: `<declaration>` [DIA-5]"; `report_skipped_foreign`
    (a use of a declaration the import skipped) and `E5056`/`E5058` (a class deriving from a C++
    class: its nearest header-declared base) add it too.
  - `compiler/ember_diag/src/lib.rs` — `Diagnostic::notes_from` (an optional note).
- Ceiling: C++ member functions are not recorded separately (an error at one names its class only
  through the class); add them when a member-level FFI error needs its own line.
- Baselines: `tools/rule_index_baseline.json` still lists `E5065` under codes without pages and
  without tests; both now exist (`--write-baseline` after merging drops them).

## CG-C-7 — done (built)
- Test: `tests/conformance/CG-C-7/accept_locals_keep_their_ember_names_in_the_c.em` (a parameter, a
  local, a transliterated non-ASCII name, a C keyword and a name the function's C uses suffixed,
  two locals of one name; debug and release; checked on the emitted C).
- Compiler: every local was `_N` in the C, its Ember name only in a comment, so debuggers showed
  `_12`. `compiler/ember_codegen_c/src/lib.rs` — `name_locals`: once a function's C is written
  (`emit_body`), each local the program named is renamed in that text from `_N` to its Ember name
  (`[MNG-3]`-transliterated): parameters, declarations, uses. Members and designators (`._N`,
  `->_N`) are fields and stay. The plain name is refused, and the local is `<name>_<N>`, when C
  would read it as something else: a keyword or a standard-header macro (`C_RESERVED`, shared with
  `member`, plus `C_MACRO_NAMES`), an all-upper-case name (C's macro spelling), the runtime's
  prefixes, an identifier the function's own C uses (a function it calls, a type, a foreign name),
  any identifier a runtime macro the function uses expands to (`runtime_header_macros`, read from
  `ember_rt.h`/`ember_simd.h` at compile time, transitively), or another local's name. Names that
  are no C identifier or start with `_` (the compiler's own) keep `_N`. Prototypes keep `_N`
  (their parameter names mean nothing to C). A view parameter Windows passes as two words keeps
  `_N_p`/`_N_n` and is rebuilt under its name at entry.
- Tests updated to the names (their needles named `_1`, `_2`…; each now names the same local):
  `CELL-1` (2), `CELL-2`, `EXP-6`, `HR-10`, `HR-25` (2), `HR-5` (2), `MONO-7`, `OWN-8`, `RNG-4`,
  `SIMD-7` (6, `; ++_` -> `; ++i`), `TYP-16` (4), `TYP-9`; `milestones.rs`
  `wrapping_arithmetic_is_unsigned_in_c`, `aggregates_and_views_are_written_field_by_field`, and
  `register_field_copies` (a plain local is any C identifier now, not only `_N`).
- Checked: every conformance file's `assert-c*` lines (395 files) against the emitted C; the whole
  conformance tree through `annotations.py` with MSVC and with clang.

## CG-C-10 — done (built)
- Tests: `tests/conformance/CG-C-10/run_fail_a_backtrace_names_paths_and_places.em` (a backtrace
  through a method, a module function and two root functions, each frame `path at file:line:col`;
  MSVC and clang), `accept_frames_are_named_by_path_and_calls_by_place.em` (the tables, on the
  emitted C, MSVC/clang/gcc), support `geometry.em`; `milestones.rs`
  `demangle_writes_a_c_stack_in_ember_paths`.
- What was missing: frames printed the function's declaration line (no column) and a method by its
  bare name (`at`); there was no `ember demangle`.
- Compiler:
  - `ember_hir::Function::path`, `ember_mir::Body::path` (`Option`; set for methods in
    `ember_typeck`: `geometry.Grid.at`, a derived `clone`'s `T.clone`; lowered in `lower.rs`; `None`
    at every other constructor) — the function's Ember path where `name` is not it.
  - `compiler/ember_codegen_c/src/lib.rs` — frame names (and `[RT-12]`'s code names) use the path;
    `emit_call_sites`: outside `release`, `em_call_sites[]`, each call a program function makes to
    another (callee `""` when through a value, an interface or a virtual method) with its
    `file:line:col`; `emit_backtrace_registration` hands both tables to the runtime. And every call
    statement now gets its own `#line` (`emit_terminator`): calls had none, so the line a debugger
    or a PDB gave a call was counted on from the statement before (seen: main's call at line 31
    reported as 37). That is a `[CG-C-8]` defect fixed in passing.
  - Runtime (`runtime/ember_rt/templates/ember_rt.{h,c}.in`, regenerated): `ember_call_site`,
    `ember_backtrace_sites`; the backtrace prints the innermost frame at the panic's place, each
    other at its call of the frame below (`call_site`: a direct call of that callee, else one through
    a value; of several, the one on the frame's line when Windows' PDB gives it
    (`SymGetLineFromAddr64`), else the first). `EMBER_RUNTIME_ABI` not bumped: an entry point was
    added, nothing changed shape (as `ember_stack_names` was added).
  - `compiler/ember_driver/src/main.rs` — `ember demangle <file>`: checks the program, then copies
    standard input to standard output with each of its functions' C symbols (and `…__rl` bodies)
    written as Ember paths, whole identifiers only; listed in `--help`.
- Ceilings: the column is the call's last argument's (the call's own when it has none): the MIR
  keeps one span per call terminator, the last argument's, which the move/borrow diagnostics rely
  on (making it the call's span changed `tests/ui/borrow/O1` for the worse, so it was not done).
  On Linux there is no line information (no DWARF reader), so of several calls of one callee in one
  function the first is named. A function inlined (by Ember's inliner or the C compiler) is its
  caller's frame, as before.
- Spec observation: `[MNG-1]` specifies length-prefixed, injective symbols (`em_3lib1m3x_f`); the
  compiler mangles `em_` + the dotted path with `_` (`em_geometry_Grid_at`), which is not injective
  (no test names MNG-1). `ember demangle` therefore maps symbols through the program (it takes the
  program's file) rather than decoding them.

## CG-C-9 — done for what the C can carry; one ceiling for the owner (built)
- Test: `milestones.rs` `build_writes_debugger_visualisers_beside_the_program` (the three files
  beside a debug build; the natvis's entries for an enum, a struct with an `Array` and a `String`
  field, a class object without its header, `str`, `Option`; the GDB and LLDB scripts' type
  description and registration). Checked by hand: the `.natvis` parses as XML; both Python scripts
  compile; the MSVC and the clang (lld-link) builds put the `.natvis` into the program's PDB
  (`llvm-pdbutil dump -streams`: `/src/files/…/vis.natvis`). Not run in a debugger (the brief's
  rule; and this laptop's lldb lacks its Python, gdb is absent).
- What was missing: no visualisers at all.
- Compiler:
  - `compiler/ember_codegen_c/src/visualisers.rs` (new) — one description of every type the C
    defines (`describe`: structs by Ember name and fields, class objects without `_header`, tuples,
    payload enums by `tag` and variant payloads — `Option`/`Result` included —, unit enums by their
    discriminants; `Box`/`Shared`/`Weak`/`Cell`/SIMD wrappers and niche `Option`s are their payload's,
    shown as C shows them) rendered three ways: a `.natvis` (variant display strings, typed
    `ArrayItems` for `Array`/`Span` fields, `String`/`str` as text, unit-enum fields by variant), and
    GDB and LLDB Python scripts sharing a `TYPES` table (printers / summaries and synthetic children;
    a class handle shows its object; a unit enum its variant where DWARF keeps its typedef).
  - `compiler/ember_codegen_c/src/lib.rs` — `Output::visualisers`; and `cell_inner_id` no longer
    indexes a struct with no fields (`then_some` evaluated `fields[0]` eagerly: the describer hit it).
  - `compiler/ember_driver/src/main.rs` — writes `<name>.natvis`, `<program>-gdb.py` (GDB's
    auto-load name) and `<name>.lldb.py` beside the program before it is linked.
  - `compiler/ember_build/src/lib.rs` — `natvis_beside`: outside `release` the Windows link adds
    `/NATVIS:` (MSVC, clang-cl's lld-link, clang's `-Wl,`), after the program cache's key, and the
    key takes the `.natvis`'s content (a program cached without one is not served for it).
- Ceiling (owner question): one C struct serves every `Array[T]` and `String` (`ember_vec`) and
  every view (`ember_span`), so a collection is shown by its elements only where its Ember type is
  known: a field of a struct, a class or a variant. One held directly in a local shows its length.
  A typedef per instance would carry the type for GDB and LLDB (DWARF keeps typedefs) but not for
  Visual Studio (CodeView records a local's underlying type: checked with `llvm-readobj --codeview`).
  Options: (a) a C struct per collection instance (`struct em_Array_i64 { ember_vec v; }`, passed
  to the runtime as `&x.v`) — every value shown typed everywhere, a large backend change; (b) a
  typedef per instance — typed locals in GDB/LLDB only, small; (c) leave as is. Recommendation: (a)
  when the backend is next reworked; (b) is cheap but helps only Linux/macOS debuggers.

## OPT-1 — done (built)
- Tests: `tests/conformance/OPT-1/accept_an_object_no_handle_outlives_is_made_in_the_frame.em` (a
  class object made, used through a call and dropped each turn of a loop is in the frame, its
  `drop` printing as before; one pushed onto a list and one a variable keeps past its turn stay on
  the heap), `accept_a_handle_that_may_outlive_its_function_stays_on_the_heap.em` (through two
  calls: frame; a method returning `self`, a `Weak` made of it: heap). Debug and release, the
  emitted C checked.
- What was missing: nothing was ever promoted (a MAY, but the owner's C-speed goal and HANDOFF's
  plan named it).
- Compiler:
  - `compiler/ember_analysis/src/frame_objects.rs` (new) — `place_frame_objects_all`: escape
    analysis over MIR. Aliases of a constructed handle (copies and class upcasts into locals);
    escape = stored anywhere but a local, returned, aggregated, borrowed as the local itself,
    cast to an interface, or passed to a built-in, a callable value, a virtual/interface method, a
    foreign function or a direct call whose parameter escapes (per-parameter summaries, a greatest
    fixpoint over the call graph). A class whose `drop` (or a base's) lets `self` escape is never
    promoted (the runtime calls it, no call this analysis follows). Reuse in a loop: backward
    liveness over the aliases; at the constructor no alias holding the earlier object may be live
    (MIR drops a counted handle explicitly before it is overwritten, a use). Not in a function
    that can come back to itself (`stack_lists::may_come_back`, now `pub(crate)`), at most 256
    bytes of fields, at most four a function, not in closures or reloadable builds.
  - `ember_mir::Body::frame_objects` (all constructors), the pass run last for C in
    `compiler/ember_driver/src/main.rs`.
  - `compiler/ember_codegen_c/src/lib.rs` — the object's own C struct in the frame (`_fo<N>`) and
    `ember_obj_new_frame(&_fo<N>, &em_ti_X)` for its constructor.
  - Runtime templates (regenerated): `EMBER_OBJ_FRAME`, `ember_obj_new_frame` (the heap path's
    header, counts, exclusivity words and debug tracking), and the last weak release untracks a
    frame object and frees nothing.
- Checked: the whole conformance tree through `annotations.py` with MSVC and with clang.
- Speed: changes allocation in programs with short-lived class objects: the laptop bench should
  run before pushing.

## VER-3 — not done: blocked on a spec question
- Today `@deprecated(…)` is `E0900` (`[ATT-6]`: an attribute whose effect is not built is
  refused). Building it means: accept `@deprecated(since = "…", note = "…")` (the one form, F-020)
  on any item, and warn at each use of the item outside its own module, naming `since` and `note`.
- The question: that warning needs a code, and `[DIA-6a]` requires every code the compiler emits
  to be one the specification names (registry, §XVII.9); none is named for a deprecated use.
  Options: (a) a new warning `W2192` "use of a deprecated item", defined by `[VER-3]` — every user
  of the item sees it by default; (b) a lint (`L2003`), opt-in — too quiet for a removal notice;
  (c) a note on nothing (no diagnostic) — the attribute would be documentation only. Recommendation:
  (a), with `-D warnings` making it an error where a package wants that. "Removal no earlier than
  the next major version" is a rule for whoever maintains `std` and packages, not something the
  compiler can check before versioned packages exist (`[VER-2]`, after 1.0).

## ALC-2 — not done: blocked on two spec questions
- `std.mem` has no `Allocator` interface (nor `Layout` or `Global`, which XV's table lists);
  `[ALC-1]` and `[ALC-3]` wait on it too (`docs/phase-status.json`'s `ALC-3` entry).
- Question 1 — the interface's methods: the specification names `Allocator` but gives none.
  Recommendation: `allocate(self, layout: Layout) -> *mut u8` (null on failure),
  `deallocate(self, p: *mut u8, layout: Layout)`, `reallocate(self, p: *mut u8, old: Layout,
  new_size: usize) -> *mut u8`, with `Layout { size: usize, align: usize }` and `Global` the
  runtime's `ember_alloc`/`ember_free`.
- Question 2 — how "implementing `Allocator` is `unsafe`" is written: `[TYP-43]` (ODR-111) makes
  "an `unsafe extend` of any interface but a marker" `E2040`. Options: (a) declare `pub unsafe
  interface Allocator` and let `unsafe extend A implements Allocator:` (with members) implement an
  unsafe interface, `E3100` without `unsafe` as for the markers — amending `[TYP-43]`'s sentence to
  "any interface but a marker or an `unsafe interface`"; (b) keep `extend` safe and make each
  method `unsafe fn` — but then implementing is not unsafe, only calling, which is not what ALC-2
  says. Recommendation: (a), as Rust's `unsafe impl GlobalAlloc`.
- Tests changed with it: `OBJ-4/accept_objects_are_made_by_the_runtime_allocator.em` (its point now
  sits in a list, so the heap path it is about stays exercised; a lone local point is promoted),
  `DRP-6/accept_class_handle_and_shared_release.em` (its needle names the type information, made
  either way; its subject, the release at scope exit, is unchanged).
- Spec question: `[CLS-1]` "a class instance lives on the heap" and `[OBJ-4]` "objects are allocated
  through the runtime allocator" read as absolute, while `[OPT-1]` permits the frame. Built on the
  reading that `[OPT-1]`'s MAY is the as-if exception to both (the header, counts and `drop` are
  the same; "this cannot be observed"). One thing a program can see: an embedding host's
  allocation functions (`[FFI-27]`) are not called for a promoted object. Options: (a) add "except
  as `[OPT-1]` permits" to `[CLS-1]` and `[OBJ-4]` and say in `[OPT-1]` that the host's allocator
  count is not behaviour; (b) restrict `[OPT-1]` to builds without a host allocator. Recommendation
  (a).
- Also in this commit: the CG-C-9 GDB script's typed-elements child (a cast to an array pointer,
  then a dereference: GDB does not cast a value to an array type).
