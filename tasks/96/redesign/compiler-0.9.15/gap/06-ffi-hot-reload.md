# Gap map 06: the C interface and hot reload (FFI, BLD-FFI, HR)

Agent 06, 2026-10-27. Read-only: nothing was built or run. Scratch scripts are in
`target/gap-0.9.15/06/`:
- `extract.py` prints a rule's text from both spec trees;
- `wdiff.py` gives a word diff of every changed rule (`wdiff.txt`);
- `classify.py` and `final.py` mark the old tests (`final.txt`, `nonkeep.md`).

**Scope.** The H.41 table lists 64 rules of these families:
- 54 `changed`: BLD-FFI 3, FFI 34, HR 17;
- 10 `added`: FFI-51 to FFI-56, HR-44 to HR-47;
- none `removed`.

My extraction of both trees finds exactly these 64 rules with different texts. Three more rows cover
features the compiler has under removed names: `ForeignBox`, `Retained.pin` of objects, and the run-time
re-entrancy check across C++ calls. That makes 67 rows.

**How the old tests were classified.** 308 `.em` files sit in the folders of the changed rules.
- A script marks old spellings in each file, and in the overlay files the test uses (legend in 1b).
- I read by hand every file whose meaning may change, about 50 of them: all of FFI-21, FFI-23,
  FFI-17d, FFI-33b, FFI-39c, FFI-39d, FFI-40, FFI-40a, FFI-42a, BLD-FFI-5a and HR-*, plus the E5015
  and `extern "C" fn` hits in FFI-8, FFI-10 and FFI-11.
- The rest are classified by their marks.

The convention is area 05's:
- `respell`: the same meaning, accepted or refused the same way. Only spellings change.
- `rewrite`: what is accepted or refused changes.
- `retire`: the test checks only a removed feature.

**One cross-area rule dominates the respells.** `[UNS-8]` (changed, area 05's A5-2) now asks a
`# SAFETY:` note on every `unsafe extern` and `unsafe overlay` block, with W3012 when it is missing.
Today W3012 covers only `unsafe:` blocks and `unsafe fn` (`ember_parser/src/decls.rs:990-993`,
`ember_parser/src/body.rs:215-216`). The runner fails on an unexpected warning
(`ember_driver/tests/milestones.rs:409-419`). The effect on my folders:
- 172 test files lack the note on their extern block;
- 65 overlay files under `overlays/` lack it;
- for 109 of the 254 respells, this note is the only change.

## 1. Rule table

Old tests are given as keep / respell / rewrite / retire. "—" means the rule has no folder. Design tests
are counted by `conformance-rule-map.md`. 34 more design programs exercise FFI without naming an FFI
ID (R1 1, R12 2, R15 25, R16 6); they are listed under the work items that make them pass.

| rule | H.41 kind | what the spec now requires (one line) | today in the compiler | status | what must change | old tests | design tests |
|---|---|---|---|---|---|---|---|
| BLD-FFI-1 | changed | a C source compiled in is `unsafe import c "./bridge.c"` (FFI-51) | `import c` parsed with no `unsafe` (`ember_parser/src/decls.rs:35-60`; `ember_ast/src/lib.rs:242` `ImportKind::Foreign` has no unsafe flag) | not built | A6-2 | — | 0 |
| BLD-FFI-3 | changed | each linked library is listed by `ember tcb` as an external library | one fixed line "the foreign declarations below…" (`ember_driver/src/lib.rs:1736`) | partly | A6-2: list each `link`/`[link] libs` entry | — | 0 |
| BLD-FFI-5a | changed | with the C++ header, a `thread_shared` class value crosses as a copyable handle class: each copy one atomic owner, limit checked (RT-7/RT-8); no other Ember value has a handle | only `@sync` classes cross (`ember_typeck/src/lib.rs:21073-21085`, E5015 "mark it `@sync`"); handle functions in `ember_codegen_c/src/class_exports.rs` | partly | A6-13 (checker) and A9-9 (both backends) | 0/0/2/0 | 0 |
| FFI-1 | changed | the contract also says whether the function may call back into Ember (FFI-52) | contract words `ember_typeck/src/lib.rs:60360-60364` have no `no_callbacks`; `@ffi(no_callbacks)` is E0104 (`:22259`) | partly | A6-8 | 0/5/0/0 | 0 |
| FFI-2a | changed | wording only: `const T*` a read-only view, `T*` one C may write through | derived facts built (cimport) | text only | — | 0/2/0/0 | 0 |
| FFI-8 | changed | table: C fn pointers are `unsafe extern "C" fn` (FFI-56); `owns T`, `CStrView`, `ThreadShared`; `tail(n: usize)`; a promoted variadic argument is E5013 with the new message | fn pointers mapped safe (`ember_driver/src/cimport.rs:1957-1985`); promotion is E5015 (`ember_typeck/src/lib.rs:56189-56194`); E5013 not in `ember_diag/src/codes.rs` | partly | A6-11, A6-12, A6-5, A6-1 | 10/15/1/0 | 1 (R16) |
| FFI-10 | changed | hand declarations: `@ffi(no_callbacks)`; the words in front of parameter names (`ref mut`, `owns`); `Option[MutSpan]` on a `ref mut` parameter, whose argument the call marks `ref mut` | `mut p: T` and `owned p:` modes (`check_foreign_ownership`, `ember_typeck/src/lib.rs:22146`); no `no_callbacks` | partly | A6-4, A6-8 | 2/50/0/0 | 0 |
| FFI-11 | changed | vocabulary: `owns x: owns T` for `owned`; an `owns T` result; `retained` is only `callback(retained)`, otherwise an unsafe raw pointer; `from(this)`; `ref mut` for `exclusive`; `no_callbacks`; `safe` on a typedef | `owned` gives `ForeignBox[T]`, `retained` takes `Retained[T]` (`ember_typeck/src/lib.rs:60983-61040`, `:22143-22210`) | partly | A6-4, A6-5, A6-10, A6-8 | 0/84/0/0 | 0 |
| FFI-11b | changed | `Option[MutSpan[T]]` on a `ref mut` parameter | built with a `mut` parameter | partly | A6-4 (spelling) | 0/4/0/0 | 0 |
| FFI-15 | changed | `cstr` becomes `CStrView`, `str` becomes `StrView`; "view" wording | `TyKind::CStr`; `std/src/ffi.em:19-33` (`extend str`, `to_cstring(self)`) | partly | A6-1, A6-4 (the `StrView` rename is area 01/03's) | 2/20/0/0 | 0 |
| FFI-17a | changed | `std::unique_ptr` becomes `owns T`; `std::function` with `@ffi(std_function)` is a function value; `CppShared` never `shares` | `ForeignBox[...]` and `adopt` emitted (`ember_driver/src/cimport.rs:3399-3408`) | partly | A6-5 | 3/10/0/0 | 0 |
| FFI-17d | changed | `owner="ember"`: the object ends when its last Ember owner does; `owner="foreign"`: a foreign `delete` ends it once it has been handed to C++ (FFI-39c) | `ember_cpp_object_adopt` checks the count at the delete (`runtime/ember_rt/templates/ember_rt.c.in:2496`); thunks `cimport.rs:3725-3730` | partly | A6-10 | 0/1/2/0 | 0 |
| FFI-17e | changed | wording (`ThreadShared`, "end", "view") | built | text only | — | 2/1/0/0 | 0 |
| FFI-21 | changed | checked trampoline (FFI-53); block moved; `borrowed` may use real variables within FFI-22; `retained`/`once` capture lists (E7004 message); `Retained[ForeignCallback]`; handing over counts as making its changes | trampoline `ember_codegen_c/src/lib.rs:6826-6960`: user data = the record's address; `once` frees the record with no mark (`:6909-6930`); captures `ember_typeck/src/lib.rs:52379-52420` (E7004, E3026, E3063) | partly | A6-6, A6-7, A6-8 | 0/15/3/0 | 1 (R15) |
| FFI-21a | changed | wording: "plain parameter", `ref_to_ptr(value)` | E5042 built | text only | `ref_to_ptr(ref x)` → `ref_to_ptr(x)` comes with area 01's `ref`-expression change | 0/2/0/0 | 0 |
| FFI-22 | changed | the trampoline checks `threads(main/creator)` in every profile; a `threads(any)` callback, borrowed or kept, is a `thread_shared fn()`: E7015, E7008; it changes real variables only through their own operations | attach built; thread check only under `EMBER_RT_DEBUG_ACCESS` (`ember_codegen_c/src/lib.rs:6863-6867`, release `:6934-6937`; LLVM `ember_codegen_llvm/src/ffi.rs:158-184, 256-260`); captures must be `Send` (E7004) | partly | A6-6, A6-7 | — | 10 (R12 4, R15 6) |
| FFI-23 | changed | `Retained[ForeignCallback]` is only the token of a kept callback; Ember gives objects through `owned` or `into_raw()`; C gives objects back through `adopt` | `Retained.pin` of a class or a `Box` (`std/src/ffi.em:196-228`; `ember_typeck/src/lib.rs:16659-16690`, `:18730-18741`, `:45660-45680`); `ember_obj_pin` (`ember_rt.c.in:3200`) | partly + remove | A6-10 | 0/0/0/2 | 0 |
| FFI-25 | changed | `on_panic=abort` is written inside `@unsafe(export(…))` | `@export(on_panic=abort)` (`ember_typeck/src/lib.rs:42636-42660`, `:60820`) | partly | A6-3 | 0/3/0/0 | 0 |
| FFI-26 | changed | `@unsafe(export(…))`/`@unsafe(export_table(…))` with a SAFETY note (W3012); the plain form is E3100; the name is optional; E0104 for `main`, the runtime prefix, the module-protocol names, C library functions and duplicates | plain `@export` (`ember_typeck/src/lib.rs:42620-42660`, `export_symbol` `:60774`); no arguments is E0900 (`:42628`); only `main` and duplicates are E0104 (`:847-855`) | partly | A6-3 | 1/8/0/0 | 0 |
| FFI-29b | changed | `implementation = [...]` only in an `unsafe import c` | accepted without `unsafe` | not built | A6-2 | 0/2/0/0 | 0 |
| FFI-30c | changed | another package's overlay is applied only to the header it was written for (FFI-55) | composition built; no hash check | partly | A6-16 | 0/2/0/0 | 0 |
| FFI-31b | changed | no owning value crosses (`owns`, `shares`, `thread_shared`, `weakref`, function values…), except a `thread_shared` class with `--cpp`; `char` is E5054 | `exported_owner_reachable` (`ember_typeck/src/lib.rs:21400-21420`); `char` counts as FFI-safe (`ember_types/src/lib.rs:2755-2758`) | partly | A6-12, A6-13 | 3/1/0/0 | 0 |
| FFI-32 | changed | a C++ class returned by value is an `owns T` | `ForeignBox` (ODR-166) | partly | A6-5 | 2/1/0/0 | 0 |
| FFI-33 | changed | a `threads=any` export reads only `ThreadShared` statics and reaches a `static mut` only under its lock (E7012); the same under `creator` | no check of the statics an export reaches (no use of `ExportThreadPolicy` in typeck or analysis beyond defaults); E7012 not registered | partly | A6-14 (with A5-18) | 1/6/0/0 | 0 (+2 R15 by content) |
| FFI-33a | changed | no one-thread owner reaches C (compile time); the run-time thread checks of 33b, 33c and 53 run in every profile; the debug check of non-`@sync` retains goes | `EMBER_OBJ_CHECK_THREAD` in debug (`runtime/ember_rt/templates/ember_rt.h.in:1522-1529`); test `ember_build/src/lib.rs:2940-2966` | partly | A6-14 | — | 0 |
| FFI-33b | changed | an `owns T` foreign value of a `threads(creator)` type records its thread and stops in every profile when it ends on another, naming the type, the making line and both threads | checked only under `EMBER_RT_DEBUG_ACCESS` (`ember_rt.c.in:2552-2567`); the message names an address, not a type or a line | partly | A6-5, A6-14 | 0/1/1/0 | 0 |
| FFI-33c | changed | wording ("the module's `static mut`s") | built (`ember_codegen_c/src/lib.rs:7272-7280`) | text only | — | — | 0 |
| FFI-36 | changed | `adopt` gives an `owns T` | `ForeignBox` | partly | A6-5 | 0/1/0/0 | 0 |
| FFI-36a | changed | `adopt[T](h) -> owns T`; its end calls `destroys(f)` once | `std/src/ffi.em:155-195`; `ember_typeck/src/lib.rs:16470-16580` | partly | A6-5 | 0/4/0/0 | 0 |
| FFI-37d | changed | `no_callbacks` and thread facts are asserted, never `instrumented` | no such fact | not built | A6-8 | — | 0 |
| FFI-39 | changed | the constructor is `fn init(…)` with no `self` | `fn init(self, …)`; the importer writes receivers | partly | A1-7 (`this`), A6-1 | 0/9/0/0 | 0 |
| FFI-39c | changed | a keeping C++ parameter takes an `owner="foreign"` object by hand-over, or `into_raw()` in `unsafe:`; an `owner="ember"` object there is E5060 with both lines | `Retained` token upcast and E5060 "nothing holds the token" (`ember_typeck/src/lib.rs:35398-35423`, `:17334-17340`) | not built (+ remove the token path) | A6-10 | 0/0/2/0 | 0 |
| FFI-39d | changed | re-entry from C++ is checked when compiling through FFI-52 (E3020/E3021/E3022/E3060); nothing runs | run-time access checks, plus `L3013` (`ember_analysis/src/long_access_lint.rs:43, 82, 127`) | not built (+ remove) | A6-8 | 0/1/1/0 | 0 |
| FFI-40 | changed | the object of an imported method is `this`; a `const` method reads only (FFI-40a), any other changes it (E3023); `T&` is `ref mut x: T`, `const T&` is `x: T`, `T&&` is `owns x: T`; a `T&` result is a `ref mut T` view | importer writes `mut self`/`self` receivers (`ember_driver/src/cimport.rs:3637`, `:3121-3128`) | partly | A6-15 | 0/1/1/0 | 0 |
| FFI-40a | changed | import as "only reads" or "changes"; an overlay method line with `@ffi(const)` asserts "only reads" | body analysis built (`cimport.rs:794-880`); the overlay line's receiver wins | partly | A6-15 | 0/2/2/0 | 0 |
| FFI-41 | changed | static member functions are `static fn`s; `ThreadShared` | associated functions | partly (spelling) | A6-15 | 4/0/0/0 | 0 |
| FFI-42a | changed | a loop over a C++ container refuses a changing method (E3020); `const` reads are allowed | borrow rules give E3020/E3021 | partly | A6-15 (with area 02's loop rule) | 0/1/1/0 | 0 |
| FFI-51 | added | C compiled in is `unsafe import c` with a SAFETY note (W3012); without it E3100 with the given message; a header alone needs no word | none | not built | A6-2 | — | 0 (11 design programs must respell to it) |
| FFI-52 | added | a foreign call counts as running every Ember function C can reach (kept callbacks, exports, C++ overrides, live borrowed callbacks, `extern "C" fn` values) and makes their changes; `@ffi(no_callbacks)`; E5069; E3020 names the mark; recorded in interface files | none | not built | A6-8 | — | 2 (R11 1, R15 1) |
| FFI-53 | added | checked trampoline: tagged number (slot low 48 bits, generation top 16), slot bounds, generation + type in one compare, thread check, `once` used-up mark, running mark, retired slots, self-ending token; three stop texts | a trampoline without any of these (see FFI-21) | partly | A6-6 | — | 0 (6 R15 by content) |
| FFI-54 | added | hand-over to an `owned` parameter with no word; `into_raw()` only in `unsafe:` (E3100) gives `*mut T`, `*mut u8` or `ObjectPtr[C]` | `ForeignBox.into_raw` in std through `mem.forget` (`std/src/ffi.em:177-181`) | not built | A1-11/A5-4 (core), A6-9 (FFI parts) | — | 4 (R15 1, R16 3) |
| FFI-55 | added | an overlay of another package records its header's hash; a mismatch or a missing hash is E5010 | `parse_overlay` takes no `with (...)` (`ember_parser/src/decls.rs:650-688`); `.embind` hash kept for C++ only (`cimport.rs:5593`, `:5894`) | not built | A6-16 | — | 0 |
| FFI-56 | added | a fn pointer that comes from C imports as `unsafe extern "C" fn`; `@ffi(safe)` on a typedef entry, scalars and `@layout(c)` only (E5002); listed by tcb; an `owns T` end calls the generated `destroys(f)` | mapped as a safe `extern "C" fn` (`cimport.rs:1957-1985`) | not built | A6-11 | — | 0 |
| HR-2 | changed | wording ("changed, handed over or ended") | built | text only | — | — | 0 |
| HR-2a | changed | plan also reads HR-44's facts; a removed field's `drop` runs | schemas and the registered set | partly | A6-20 | — | 0 |
| HR-3b | changed | frame records also list the functions the rest of the function may still call | type dependencies only (`ember_codegen_c/src/frames.rs:1-12`) | partly | A6-20 | — | 0 |
| HR-6a | changed | a function value's "code word" | built | text only | — | — | 0 |
| HR-8 | changed | the manifest also records HR-44's facts | manifest `ember_codegen_c/src/reload.rs:482-612` | partly | A6-20 | — | 0 |
| HR-10 | changed | `@no_alias_checks` named in place of `@static_safe` | built | text only | — | 1/0/0/0 | 0 |
| HR-11 | changed | a field's schema includes its type's words (`owns`, `shares`, …, `using A`) | schema hash `ember_codegen_c/src/reload.rs:344-420` | partly | A6-18 | — | 0 |
| HR-12 | changed | the reloadable header is 24 bytes (8 + 16), or 28 (12 + 16) where `weakref` links reach | 40/24 (`ember_codegen_c/src/reload.rs:14`, witness `ember_obj_header_v40`) | partly | A6-19 = A9-8 (needs area 03's header) | 0/0/1/0 | 0 |
| HR-14 | changed | new rows: `gen fn` frames (HR-45), `using` values (HR-46), `@cached` (HR-44), a change of a type's words | old rows built | partly | A6-20, A6-21, A6-22 | 2/0/0/0 | 0 |
| HR-15 | changed | rewrite every owner, link and kept view (`owns`, `shares`, `thread_shared`, `weakref`, `ref`, `Span`, `StrView`…) in instances, containers, statics, `owns` blocks and function state; and the `using` instance address | field kinds `runtime/ember_rt/templates/ember_rt.h.in:3082-3096` (HANDLE, WEAK, BOX, CALLABLE…) | partly | A6-18 | — | 0 |
| HR-15a | changed | what C holds (HR-20) is never rewritten; the reload is refused when it must move | pinned only through `Retained.pin` (`ember_rt.c.in:3200`) | partly | A6-18 with A6-9 | — | 0 |
| HR-16 | changed | `static fn migrate_from(old: OldSelf)`; `old` is a read-only view | `fn migrate_from(old: ref OldSelf)` (`ember_driver/src/reload.rs:433-440`, `:628-640`) | partly | A6-17 | — (HR-35's 2) | 0 |
| HR-17a | changed | wording ("ends") | built | text only | — | — | 0 |
| HR-19 | changed | the bodies tier also records HR-44's facts and HR-45's frame schemas | records neither | partly | A6-20, A6-21 | 0/0/1/0 | 0 |
| HR-20 | changed | objects C holds (handed to `owned`, or `into_raw()`) and kept callbacks' state never move; a reload refused names it; `on_relocate` is gone | `Retained.pin` pins; `on_relocate` rows (`ember_rt.c.in:3212-3270`; `ember_codegen_c/src/lib.rs:9769-9794`) | partly + remove | A6-10, A6-18 | 0/0/0/1 | 0 |
| HR-35 | changed | a cast and a count at its limit inside `migrate_from` also return `Err` | prepare's guard turns every panic into `ReloadError.Migration` (`ember_rt.c.in:11937-11960`) | built (the new stops inherit it if they end through `ember_panic`) | — | 0/2/0/0 | 0 |
| HR-41 | changed | matrix rows for HR-44 to HR-47 | — | partly | A6-20 to A6-22 | — | 0 |
| HR-44 | added | the inferred facts of reloadable functions are recorded and compared (E9030), naming the function, the fact and the caller; `@cached` results cleared | none | not built | A6-20 | — | 0 |
| HR-45 | added | a `gen fn` frame has a schema (layout and states); a live frame resumes in new code or the reload is refused | none | not built | A6-21 | — | 0 |
| HR-46 | added | a value made `using` an instance migrates in place or is refused; the instance address is rewritten; an `Allocator` type's schema stays fixed while its values live | none | not built | A6-22 | — | 0 |
| HR-47 | added | `@reinit_on_reload` is refused where a view, a `sharedref mut` or an instance address may point into the static | none | not built | A6-22 | — | 0 |
| *(removed)* `ForeignBox[T]` and `ForeignBox.into_raw` | removed name | gone: an `owns T` of a foreign type; the compiler's own `into_raw()` | `std/src/ffi.em:165-195`; `ember_typeck/src/lib.rs:16470-16580`, `:50186-50203`; `ember_codegen_c/src/lib.rs:6631-6760`; `ember_codegen_llvm/src/ffi.rs`; `cimport.rs` (14 sites) | remove | A6-5 | (inside FFI-36a and others) | 0 |
| *(removed)* `Retained.pin` of a class or a `Box`, `retained` object contracts, `on_relocate` | removed name | gone (FFI-23, HR-20, verdict 91) | as FFI-23 and HR-20; `Builtin::RetainedPin`/`RetainedUnpin` (`ember_hir/src/lib.rs:1742-1748`); `ember_interp/src/builtin.rs:1474` | remove | A6-10 | (FFI-23, HR-20) | 0 |
| *(removed)* the run-time re-entrant access check and `L3013` across C++ calls (EXC-7) | removed name | gone: compile time through FFI-52 | `ember_analysis/src/long_access_lint.rs:43, 127`; the EXC access words | remove | A6-8 (with area 04's EXC removal) | (FFI-39d) | 0 |

Status: built 1, partly 41, not built 14, remove 3, text only 8 (67 rows).

Old tests in these folders (308 `.em`): keep 33, respell 254, rewrite 18, retire 3. In the Rust tests:
- `ember_driver/tests/reload.rs:1217` (`a_pinned_object_is_never_moved`): rewrite with `into_raw()`;
- `ember_driver/tests/reload.rs:1246` (`a_pinned_object_with_on_relocate_moves_and_is_told`): retire;
- `ember_build/src/lib.rs:2940` (FFI-33a's debug retain check): retire.

### 1b. Old tests by rule (every non-keep named)

Legend for the respell marks:
- `note`: add `# SAFETY:` above `unsafe extern`/`unsafe overlay` (UNS-8);
- `ovnote`: the same in an overlay file the test uses;
- `importc`: `import c "x.c"` or `implementation=` becomes `unsafe import c` with a note (FFI-51);
- `export`: `@export(…)`/`@export_table(…)` becomes `@unsafe(…)` with a note (FFI-26);
- `self`, `init`: `self`/`mut self` becomes `this`, or no receiver;
- `owns`: `owned` becomes `owns`, and `owned fn` becomes capture words;
- `fbox`: `ForeignBox[T]` becomes `owns T`;
- `get`: `.get()`/`.get_mut()` on a foreign owner becomes a direct call;
- `mut`: a `mut p: T` parameter becomes `ref mut p: T`, and the call says `ref mut`;
- `refp`: `p: ref T` becomes `p: T`;
- `refa`: a `ref x` argument becomes `x`;
- `cstr`, `str`: become `CStrView`, `StrView`;
- `sync`: becomes `ThreadShared`;
- `mig`: becomes `static fn migrate_from(old: OldSelf)`.

Keep (33) is every file of the folder not named below.

- **BLD-FFI-5a**
  - rewrite (2): `accept_a_sync_class_object_crosses_an_export.em` (@sync class -> a `thread_shared` value crosses (BLD-FFI-5a, FFI-31b)); `reject_an_object_of_a_class_that_is_not_sync.em` (refusal is now "not a thread_shared value", help no longer `@sync` (E0104 for @sync))
- **FFI-1**
  - respell (5): `accept_a_declaration_states_its_effects.em` (importc); `accept_a_thread_rule_is_part_of_the_contract.em` (importc note); `reject_a_declaration_that_states_nothing_has_every_effect.em` (importc); `reject_a_function_has_no_creator.em` (note); `run_fail_a_main_thread_function_called_elsewhere.em` (owns importc note)
- **FFI-2a**
  - respell (2): `accept_const_is_a_shared_borrow_and_its_absence_a_mutable_one.em` (ovnote); `reject_a_const_pointer_is_never_a_mutable_borrow.em` (ovnote)
- **FFI-8**
  - rewrite (1): `reject_variadic_calls_are_unsafe_and_typed.em` (promoted further arguments are E5013 with the new message, not E5015)
  - respell (15): `accept_a_c_enum_holds_any_value.em` (str note); `accept_a_cpp_alias_is_a_newtype.em` (get ovnote); `accept_a_flexible_array_member_is_reached_by_tail.em` (importc refp get ovnote); `accept_a_newtype_compares_hashes_and_shows_as_its_representation.em` (str); `accept_a_newtype_is_nominal_with_its_typedefs_representation.em` (importc note ovnote); `accept_atomic_is_std_sync_atomic.em` (refa); `accept_long_double_is_c_longdouble.em` (importc ovnote); `accept_std_ffi_c_aliases_are_the_targets_c_types.em` (note); `accept_unions_are_layout_c_unions.em` (ovnote); `accept_variadic_calls_and_va_list.em` (owns importc refp refa ovnote); `accept_volatile_pointers_are_volatile.em` (refa); `reject_a_c_enum_match_without_a_wildcard.em` (str note); `reject_a_flexible_struct_is_never_by_value.em` (refp); `reject_a_newtype_entry_names_a_typedef_alone.em` (importc ovnote); `reject_a_va_list_is_passed_on_once.em` (owns importc ovnote)
- **FFI-10**
  - respell (50): `accept_borrowed_aggregate_at_manual_boundary.em` (note); `accept_foreign_call_inside_unsafe_function.em` (note); `accept_foreign_record_function_values.em` (owns note); `accept_foreign_record_statics.em` (note); `accept_foreign_scalar_statics.em` (note); `accept_foreign_static_syntax.em` (note); `accept_foreign_statics_as_places.em` (note); `accept_globals_holding_pointers.em` (importc note); `accept_opaque_foreign_handle.em` (note); `accept_safe_borrowed_one_mut_parameter.em` (mut note); `accept_safe_foreign_function_value.em` (note); `accept_safe_manual_c_function.em` (note); `accept_safe_mut_parameter_with_link_name.em` (mut note); `accept_safe_shared_one_reference.em` (refp note); `accept_system_and_fastcall_declaration_blocks.em` (note); `accept_two_shared_pointer_contracts.em` (refp note); `accept_unsafe_manual_c_call.em` (note); `accept_vectorcall_declaration_block.em` (note); `reject_array_of_opaque_foreign_size.em` (note); `reject_bool_foreign_static_augmented_write.em` (note); `reject_borrowed_foreign_record_static.em` (cstr str sync note); `reject_call_without_unsafe.em` (note); `reject_contract_for_unknown_parameter.em` (note); `reject_contract_without_pointer_carrier.em` (note); `reject_defined_type_in_foreign_block.em` (note); `reject_duplicate_foreign_name.em` (note); `reject_duplicate_pointer_contract.em` (mut note); `reject_foreign_mutable_static_read_without_unsafe.em` (note); `reject_foreign_record_field_write_without_unsafe.em` (note); `reject_foreign_static_conflicting_aliases.em` (note); `reject_foreign_static_initializer.em` (note); `reject_foreign_static_read_without_unsafe.em` (note); `reject_foreign_static_unsupported_type.em` (str note); `reject_foreign_static_write_without_unsafe.em` (note); `reject_generic_opaque_foreign_field.em` (note); `reject_immutable_mutable_foreign_static.em` (note); `reject_nested_foreign_field_write_without_unsafe.em` (note); `reject_non_c_manual_abi.em` (note); `reject_noncopy_foreign_record_static.em` (self note); `reject_opaque_foreign_by_value.em` (note); `reject_opaque_foreign_field.em` (note); `reject_opaque_foreign_size.em` (note); `reject_safe_mut_parameter_without_contract.em` (mut note); `reject_safe_pointer_without_contract.em` (note); `reject_safe_raw_pointer_with_contract.em` (note); `reject_safe_shared_ref_without_contract.em` (refp note); `reject_write_to_immutable_foreign_record_field.em` (note); `reject_write_to_immutable_foreign_static.em` (note); `reject_write_to_immutable_nested_foreign_field.em` (note); `reject_writing_an_element_of_a_foreign_static_array.em` (note)
- **FFI-11**
  - respell (84): `accept_an_opaque_handle_is_borrowed_as_one.em` (importc refp get ovnote); `accept_borrowed_result_from_named_parameter.em` (refp refa note); `accept_contract_words_in_any_order.em` (cstr mut refa note); `accept_count_witness_before_pointer_and_scalar.em` (note); `accept_counted_mutable_span_call_c.em` (note); `accept_counted_result_from_input_span.em` (note); `accept_counted_result_from_mutable_input_span.em` (note); `accept_counted_result_from_nullable_input.em` (note); `accept_counted_shared_spans_call_c.em` (note); `accept_counted_span_with_borrowed_result.em` (refp note); `accept_counted_span_with_u8_witness.em` (note); `accept_exclusive_counted_result_from_mutable_input.em` (note); `accept_exclusive_counted_static_result.em` (note); `accept_exclusive_fixed_static_result.em` (note); `accept_exclusive_nullable_fixed_result_from_parameter.em` (refa note); `accept_exclusive_one_nullable_result_from_parameter.em` (refa note); `accept_exclusive_one_nullable_static_result.em` (note); `accept_exclusive_one_result_from_parameter.em` (refa note); `accept_exclusive_one_static_result.em` (note); `accept_explicit_aliased_fixed_result.em` (note); `accept_explicit_aliased_one_reference.em` (refp note); `accept_explicit_aliased_shared_result.em` (note); `accept_fixed_array_with_borrowed_result.em` (refp refa note); `accept_fixed_array_with_counted_span.em` (refp refa note); `accept_fixed_borrowed_array_call_c.em` (refp refa note); `accept_fixed_empty_array_call_c.em` (refp refa note); `accept_fixed_mutable_array_call_c.em` (refa note); `accept_fixed_result_from_array_parameter.em` (refp refa note); `accept_nullable_aliased_one_reference.em` (note); `accept_nullable_counted_empty_result_is_some.em` (note); `accept_nullable_counted_mutable_input_call_c.em` (note); `accept_nullable_counted_result_from_input_span.em` (note); `accept_nullable_counted_shared_input_call_c.em` (note); `accept_nullable_counted_shared_witness.em` (note); `accept_nullable_exclusive_counted_empty_result_is_some.em` (note); `accept_nullable_exclusive_counted_result.em` (note); `accept_nullable_fixed_array_call_c.em` (refa note); `accept_nullable_fixed_mutable_array.em` (refa note); `accept_nullable_fixed_result_from_parameter.em` (refp refa note); `accept_nullable_mut_one_reference.em` (note); `accept_nullable_result_from_named_parameter.em` (refp refa note); `accept_nullable_shared_one_reference.em` (note); `accept_nullable_static_borrowed_one_result.em` (note); `accept_nullable_static_fixed_array_result.em` (note); `accept_nullable_static_result_some_and_none.em` (note); `accept_out_parameters_are_returned.em` (mut note); `accept_out_parameters_beside_a_contracted_result.em` (importc ovnote); `accept_static_borrowed_one_pointer_result.em` (note); `accept_static_borrowed_one_result_call_c.em` (note); `accept_static_counted_result_outlives_input.em` (note); `accept_static_fixed_array_result.em` (refp note); `accept_static_result_outlives_input_borrow.em` (refp refa note); `accept_two_fixed_array_parameters.em` (refp refa note); `reject_a_contract_left_unsafe_asks_for_no_fact.em` (refa note); `reject_a_handle_result_says_who_owns_it.em` (note); `reject_borrowed_pointer_result_without_contract.em` (note); `reject_count_witness_passed_by_mutable_reference.em` (mut note); `reject_counted_result_unequal_input_lengths.em` (note); `reject_counted_result_without_input_span.em` (note); `reject_counted_shared_spans_with_unequal_lengths.em` (note); `reject_counted_span_length_that_overflows_u8.em` (note); `reject_counted_span_with_noninteger_witness.em` (note); `reject_counted_span_without_witness.em` (note); `reject_exclusive_and_aliased_result.em` (note); `reject_exclusive_counted_result_shared_type.em` (note); `reject_exclusive_counted_result_without_input_span.em` (note); `reject_exclusive_fixed_result_shared_type.em` (note); `reject_exclusive_one_result_shared_type.em` (note); `reject_exclusive_one_result_without_exclusive_contract.em` (note); `reject_fixed_array_length_mismatch.em` (refp note); `reject_fixed_array_mutability_mismatch.em` (refp note); `reject_fixed_contract_on_span.em` (note); `reject_fixed_result_length_mismatch.em` (note); `reject_fixed_shared_contract_on_mutable_array.em` (note); `reject_nonnullable_count_contract_on_optional_span.em` (note); `reject_nullable_counted_shared_witness_length_mismatch.em` (note); `reject_nullable_exclusive_counted_result_shared_type.em` (note); `reject_nullable_fixed_array_length_mismatch.em` (note); `reject_nullable_fixed_result_wrong_shape.em` (note); `reject_nullable_mut_reference_without_contract.em` (note); `reject_nullable_reference_without_contract.em` (note); `reject_nullable_result_contract_on_nonnullable_reference.em` (note); `reject_result_lifetime_from_copy_parameter.em` (note); `reject_result_lifetime_from_unknown_parameter.em` (refp note)
- **FFI-11b**
  - respell (4): `accept_inout_count_beside_out_and_status.em` (cstr mut note); `accept_inout_count_enumerates_in_two_calls.em` (mut note); `accept_inout_count_with_a_result_returns_both.em` (mut note); `reject_inout_count_misuse.em` (mut note)
- **FFI-15**
  - respell (20): `accept_cstr_literal_at_c_boundary.em` (cstr note); `accept_cstr_literal_constant.em` (cstr note); `accept_cstr_result_from_parameter.em` (cstr note); `accept_cstr_static_result.em` (cstr note); `accept_cstr_to_str_result_borrows_source.em` (cstr str); `accept_nullable_cstr_input_call_c.em` (cstr note); `accept_nullable_cstr_result_from_input.em` (cstr note); `accept_nullable_cstr_result_from_nullable_input.em` (cstr note); `accept_nullable_cstr_static_result.em` (cstr note); `accept_string_to_cstring_read_through.em` (cstr note); `accept_to_cstring_owns_terminated_bytes.em` (cstr note); `reject_cstr_outlives_cstring_owner.em` (cstr); `reject_cstring_implicit_cstr_coercion.em` (cstr note); `reject_nullable_cstr_input_contract_mismatch.em` (cstr note); `reject_nullable_cstr_result_contract_mismatch.em` (cstr note); `reject_nullable_cstr_without_contract.em` (cstr note); `reject_str_as_cstr.em` (cstr note); `reject_text_view_outlives_cstring_owner.em` (str); `reject_uncontracted_cstr.em` (cstr note); `reject_uncontracted_cstr_result.em` (cstr note)
- **FFI-17a**
  - respell (10): `accept_a_std_function_crosses_as_a_callable.em` (ovnote); `accept_a_unique_ptr_with_its_own_deleter.em` (get); `accept_a_variant_crosses_as_an_enum.em` (ovnote); `accept_fixed_spans_cross_as_arrays.em` (refa get ovnote); `accept_standard_library_types_cross_as_bridge_types.em` (get); `accept_views_and_optional_values_cross_safely.em` (get ovnote); `accept_views_cross_in_unsafe_without_an_overlay.em` (get); `reject_a_variant_the_overlay_does_not_name_whole.em` (ovnote); `reject_a_view_cannot_outlive_what_it_borrows.em` (get ovnote); `reject_a_view_result_without_a_lifetime_word.em` (ovnote)
- **FFI-17d**
  - rewrite (2): `accept_owner_foreign_lets_cpp_delete_the_object.em` (owner="foreign": the object is handed over (`owns`), no count/token (FFI-17d, FFI-39c)); `run_fail_a_foreign_delete_of_an_object_ember_still_holds.em` (an owner="foreign" object handed over leaves no Ember owner: the run-time case becomes compile-time)
  - respell (1): `run_fail_a_foreign_delete_of_an_object_ember_owns.em` (self ovnote)
- **FFI-17e**
  - respell (1): `reject_a_bridge_value_is_not_send.em` (owns)
- **FFI-21**
  - rewrite (3): `reject_cpp_callback_contracts.em` (E7004 "not Send" for a captured class object: a one-owner object is Send now; threads(any) callbacks are `thread_shared fn()` (E7015/E7008)); `reject_user_data_places_and_nullable_callbacks.em` (the E7004 line (a callable value of unknown captures) becomes the thread_shared fn() rule (E7015)); `reject_what_a_callback_captures.em` (borrowed threads(any) callback changing a real variable is E7015; a kept callback using a name with no word is E7004 (was E3026); `owned fn` captures)
  - respell (15): `accept_a_borrowed_callback_takes_a_lambda.em` (importc note); `accept_a_cpp_callback_with_user_data.em` (owns get ovnote); `accept_a_nullable_callback_is_an_option_of_a_callable_value.em` (owns importc note); `accept_adapter_names_avoid_exports.em` (export); `accept_callback_contracts_through_an_overlay.em` (owns importc ovnote); `accept_callback_returns_function_pointer.em` (str); `accept_callbacks_c_keeps_past_the_call.em` (owns importc ovnote); `accept_cpp_callback_contracts.em` (owns get ovnote); `accept_export_header_api.em` (str export note); `accept_header_native_callable_input.em` (export); `accept_native_callback_adapters.em` (export mut refa note); `accept_stored_native_function_conversion.em` (owns); `accept_user_data_named_by_its_place.em` (importc note); `reject_callback_contract_misuse.em` (note); `run_fail_a_callable_value_holding_state_has_no_c_entry.em` (owns)
- **FFI-21a**
  - respell (2): `accept_what_a_function_c_may_call_can_return.em` (refa); `reject_a_function_c_may_call_returns_a_pointer_into_its_copied_parameter.em` (refa)
- **FFI-23**
  - retire (2): `accept_retained_lets_foreign_code_keep_ember_memory.em` (Retained.pin of a class object and a Box, `retained` object contract: removed (FFI-23, verdict 91); FFI-54 replaces it); `reject_retained_misuse.em` (every line tests Retained of objects/Box and `retained` object params, removed (FFI-23))
- **FFI-25**
  - respell (3): `accept_explicit_abort_policy.em` (export); `accept_exported_panic_aborts.em` (export note); `reject_non_abort_export_policy.em` (export)
- **FFI-26**
  - respell (8): `accept_an_export_symbol_that_is_not_a_c_identifier.em` (export); `accept_an_export_table.em` (export note); `accept_an_exported_extern_c_fn.em` (export); `accept_stable_c_symbol_from_ordinary_function.em` (export); `reject_an_export_table_field_without_its_function.em` (export); `reject_duplicate_export_symbols.em` (export); `reject_export_named_as_program_entry.em` (export); `reject_unsupported_export_forms.em` (export)
- **FFI-29b**
  - respell (2): `accept_the_implementation_is_compiled_once.em` (importc); `reject_the_implementation_requested_twice.em` (importc)
- **FFI-30c**
  - respell (2): `accept_overlays_compose_in_order.em` (ovnote); `reject_overlays_that_clash.em` (ovnote)
- **FFI-31b**
  - respell (1): `reject_exported_drop_glue_and_callback_owner.em` (self)
- **FFI-32**
  - respell (1): `accept_classes_by_value_and_opaque.em` (get)
- **FFI-33**
  - respell (6): `accept_creator_thread_policy.em` (export); `accept_explicit_any_export.em` (export); `accept_explicit_main_export.em` (export); `accept_file_default_creator.em` (export); `accept_file_default_override_any.em` (export); `reject_invalid_export_threads.em` (export)
- **FFI-33b**
  - rewrite (1): `run_fail_a_creator_callback_called_on_another_thread.em` (checked in every profile (not debug only), and the stop text is FFI-53's)
  - respell (1): `accept_a_creator_callback_on_its_thread.em` (importc ovnote)
- **FFI-36**
  - respell (1): `accept_a_raw_pointer_result_is_adopted.em` (get)
- **FFI-36a**
  - respell (4): `accept_adopt_frees_with_the_declared_destructor.em` (fbox note); `accept_returns_owned_and_owned_parameters.em` (owns fbox note); `reject_adopt_without_a_destructor.em` (note); `reject_ownership_contract_mismatches.em` (owns fbox note)
- **FFI-39**
  - respell (9): `accept_a_class_derives_through_another_module.em` (self); `accept_an_ember_class_derives_from_a_cpp_class.em` (self init ovnote); `accept_an_ember_class_derives_with_no_overlay_entry.em` (self init); `accept_overrides_beyond_an_overlay_entry_are_passed_on.em` (self ovnote); `reject_a_class_that_does_not_implement_a_pure_virtual.em` (self); `reject_a_cpp_base_that_cannot_be_derived_from.em` (self); `reject_a_cpp_base_with_no_default_constructor_needs_init.em` (self init ovnote); `reject_a_trampoline_base_without_a_virtual_destructor.em` (ovnote); `reject_virtuals_names_a_method_that_is_not_virtual.em` (ovnote)
- **FFI-39c**
  - rewrite (2): `accept_cpp_keeps_an_object_through_its_token.em` (no Retained token: owner="foreign" class, object handed over (FFI-39c)); `reject_cpp_keeps_an_object_no_token_holds.em` (E5060 now means an owner="ember" object given to a keeping parameter)
- **FFI-39d**
  - rewrite (1): `run_fail_a_conflicting_reentrant_access_panics.em` (no run-time access check and no L3013: refused when compiling (E3020/E3021/E3022/E3060) through FFI-52)
  - respell (1): `accept_a_reentrant_call_reaches_the_same_object.em` (self ovnote)
- **FFI-40**
  - rewrite (1): `reject_a_reference_result_borrows_the_receiver.em` (a `T&` result is a `ref mut T` view into the object; the receiver is `this`, the expectation is restated in the view rules)
  - respell (1): `accept_methods_and_references.em` (refp get)
- **FFI-40a**
  - rewrite (2): `reject_a_const_method_of_mutable_state_through_a_shared_borrow.em` (a changing method called through a read-only name is E3023 (was E3021)); `reject_a_method_line_wins_over_the_body.em` (the overlay line asserts with `@ffi(const)`; receiver words are gone; E3023)
  - respell (2): `accept_a_class_with_mutable_state_has_no_shared_methods.em` (get); `reject_a_method_line_that_differs_from_the_header.em` (self ovnote)
- **FFI-42a**
  - rewrite (1): `reject_a_method_that_could_invalidate_the_loop_s_iterators.em` (a read-only call inside the walk is allowed now; only a changing method is E3020 (the E3021 line goes))
  - respell (1): `accept_the_loop_s_borrow_ends_with_the_loop.em` (get ovnote)
- **HR-12**
  - rewrite (1): `accept_a_reloadable_build_has_the_40_byte_header.em` (the reloadable header is 24 bytes (8 + 16) now, the witness symbol changes (HR-12, OBJ-1))
- **HR-19**
  - rewrite (1): `accept_the_bodies_tier_keeps_the_plain_header.em` (the plain header is 8 bytes now; the `ember_obj_header_v24` witness changes)
- **HR-20**
  - retire (1): `accept_a_token_with_on_relocate.em` (`Retained.pin(…, on_relocate=…)` removed (HR-20, FFI-23))
- **HR-35**
  - respell (2): `accept_migrate_from_returns_its_errors.em` (refp mig); `reject_migrate_from_may_not_panic.em` (refp mig)

## 2. Work items

Sizes: S under a day, M days, L a week or more. "Area N" names the other gap maps (01 ownership, 02
borrows, 03 the count header and the owners' types, 04 effects and closures, 05 threads and unsafe,
07 attributes, codes and codegen, 08 weakref and std, 09 helper branches).

### Slice F1: spellings at the C boundary (consistent once areas 01, 03, 04 and 05 rename their words)

**A6-1. The importer and `std.ffi` write the new words.** M.
- Files:
  - `ember_driver/src/cimport.rs` generates Ember text with the old words: about 70 string sites
    (`owned` 15, `mut self` 10, `(self` 13, `ForeignBox` 14, `cstr` 7, `Box[` 9, `Retained` 2), e.g.
    `:1979` `owned VaList`, `:3121-3128`, `:3637`, `:3745`, `:5294`, `:6767`;
  - `std/src/ffi.em`, 545 lines: `extend str`, `to_cstring(self)`, `fn drop(mut self)`,
    `fn fmt(self, mut f: …)`.
- Rules: FFI-8, FFI-15, FFI-39 (the importer's half).
- Tests: every `import c`/`import cpp` respell in 1b (the ones marked `self`/`init`/`get`).
- Depends on: A1-1 (`owns`), A1-7 (`this`), the `StrView`/`CStrView` rename (areas 01/03). It must land
  in the same commit as the parser change: a half-renamed importer breaks every `import cpp` test.

**A6-4. Hand-declaration parameter words.** M.
- Files:
  - `ember_typeck/src/lib.rs`: `check_foreign_ownership` (`:22146-22210`), the contract parsers
    (`:60900-61200`), the safe-type checks (`:21332-21420`);
  - the `from(self)` word becomes `from(this)`.
- Changes:
  - `ref mut p: T` and `ref mut p: MutSpan[T]` for `exclusive`; `x: T` and `x: [T; N]` for `borrowed`;
  - `owns x: owns T` for `owned` (FFI-11's table keeps the contract word `owned`, as area 01's Q1
    proposes);
  - the call marks `ref mut`.
- Rules: FFI-10, FFI-11, FFI-11b, FFI-15 (cstr).
- Tests: the `mut`/`refp`/`refa`/`cstr` respells: FFI-10 50, FFI-11 84, FFI-11b 4, FFI-15 20.
- Depends on: A1-8 (`ref mut` parameters and call words).

**A6-2. `unsafe import c`.** S.
- Files:
  - `ember_ast/src/lib.rs:242`: an `is_unsafe` flag on `ImportKind::Foreign`;
  - `ember_parser/src/decls.rs:35-60`;
  - the driver's C import, which compiles `.c` files and `implementation=`;
  - the `ember tcb` sections (`ember_driver/src/lib.rs:1719-1740, 2505-2530`): each `unsafe import c`
    with its note, each `link` library by name.
- E3100 with FFI-51's message for a C source or an `implementation=` import without `unsafe`; W3012
  without a note.
- Rules: BLD-FFI-1, BLD-FFI-3, FFI-29b, FFI-51.
- Tests: the `importc` respells (21 files); 11 design programs once respelled.
- Depends on: A5-2 (notes on unsafe implementations), A5-5 (tcb sections).

**A6-3. `@unsafe(export(...))` and `@unsafe(export_table(...))`.** S.
- Files:
  - `ember_typeck/src/lib.rs:42620-42680` (attribute checks), `:60774-60830` (`export_symbol`,
    `export_thread_policy`, `export_abort_policy`), `:847-855`, `:21166-21230` (tables);
  - `ember_codegen_c/src/header.rs`.
- The plain form is E3100 with FFI-26's message. The name is optional (today `@export()` is E0900).
- E0104 for:
  - `main`;
  - the runtime prefix (`ember_branding`);
  - `ember_module_init`, `ember_module_shutdown`, `ember_module_protocol_v*`;
  - a C standard library list (shared decision D6).
- Rules: FFI-25, FFI-26, FFI-33 (spelling).
- Tests: the `export` respells (21); design R15 2 once respelled.
- Depends on: area 07 (`@unsafe(...)` attribute grammar, ATT-1), A5-2.

**A6-11. C function pointers are unsafe to call.** S.
- Files:
  - `ember_driver/src/cimport.rs:1957-1985`: `unsafe extern "C" fn`;
  - the overlay entry `@ffi(safe) type PFN_x` (parser overlay items `ember_parser/src/decls.rs:693`);
  - `ember_typeck` check: scalars and `@layout(c)` only, E5002;
  - tcb line.
- Rules: FFI-56, FFI-8's row.
- Tests: none old (no test calls a pointer that came from C outside `unsafe`); add new ones.
- Depends on: nothing beyond F1.

**A6-12. `char` and promoted variadic arguments.** S.
- Files:
  - `ember_typeck/src/lib.rs:21332-21380` (E5054 for `char` in exports, hand declarations and
    `@layout(c)` aggregates, by value or behind a pointer, with FFI-31b's message);
  - `:56185-56235` (E5013 with FFI-8's message, naming the argument and the line);
  - `ember_diag/src/codes.rs` (E5013, E5054's meaning; area 07's A-item for codes).
- Rules: FFI-8, FFI-31b.
- Tests: FFI-8 1 rewrite; design R16 2.
- Depends on: area 07 (code table).

**A6-17. `migrate_from`'s first line.** S.
- Files: `ember_driver/src/reload.rs:433-440, 554-640`.
- `static fn migrate_from(old: OldSelf)`, `old` a read-only view; `@no_alias_checks` named in the HR-10
  note.
- Rules: HR-16, HR-10.
- Tests: HR-35 2 respells.
- Depends on: A1-7 (`static fn`).

### Slice F2: foreign owners (`ForeignBox` and object pinning removed in one step)

**A6-5. `owns T` of a foreign type.** M.
- Files:
  - `std/src/ffi.em:155-195` (remove `ForeignBox`; `adopt[T](h) -> owns T`);
  - `ember_typeck/src/lib.rs:16470-16580`, `:22143-22210`, `:21380-21384`, `:43412`,
    `:45874-45910`, `:50171-50203`;
  - `ember_driver/src/cimport.rs:397`, `:3399-3408`, `:4101`, `:6767`;
  - `ember_codegen_c/src/lib.rs:6631-6760, 7160-7175`; `ember_codegen_llvm/src/ffi.rs`;
  - `ember_interp`.
- Representation per D2.
- The end calls `destroys(f)` once and releases the adopt record (FFI-36b, unchanged).
- Under `threads(creator)`, the thread is checked in every profile; the stop names the type, the line
  that made the value and both threads (FFI-33b): `ember_ffi_release` (`ember_rt.c.in:2552`) leaves
  `EMBER_RT_DEBUG_ACCESS`.
- Rules: FFI-17a, FFI-32, FFI-33b (value half), FFI-36, FFI-36a, FFI-56 (last sentence), FFI-11
  (result).
- Tests: the `fbox`/`get` respells (about 19); FFI-36a 4.
- Depends on: A1-6 (`owns T`); Q2 (one destructor per type).

**A6-9. Giving an object to C.** M.
- Files: typeck call lowering for an `owned` parameter of a class type or a foreign `owns T`; the
  `into_raw()` builtin's FFI half.
- Changes:
  - releasing the adopt record of a foreign `owns T`;
  - an `ObjectPtr[C]` for a class object with one owner;
  - setting the object header's pinned flag for HR-15a/HR-20 (D4).
- Rules: FFI-54, FFI-11 (`owned`), FFI-23 (the "gives no object away" half).
- Tests: design R16 4 (`accept_a_c_function_declared_owned_takes_the_widget_with_no_word`,
  `accept_into_raw_inside_unsafe_gives_c_the_address`, `reject_a_widget_used_after_it_was_handed_to_c`,
  `reject_into_raw_outside_unsafe`); R15 `reject_implementing_send_without_unsafe` is area 05's.
- Depends on: A1-11 or A5-4 (both claim the built-in; one of them owns it), A1-9 (one-owner class
  objects), A6-5.

**A6-10. `Retained` only for callbacks; C++ keeps objects by hand-over.** M.
- Remove:
  - `Retained.pin`, `retained_pin`, `retained_unpin`, `ObjectPtr` from tokens (`std/src/ffi.em:196-268`);
  - `Builtin::RetainedPin`/`RetainedUnpin`/`RetainedAddress` (`ember_hir/src/lib.rs:1742-1748`;
    `ember_codegen_c/src/lib.rs:9769-9794`; `ember_codegen_llvm/src/builtins.rs:923`;
    `ember_interp/src/builtin.rs:1474`);
  - the `retained` object contract (`ember_typeck/src/lib.rs:22152-22175`, `:61005-61040`);
  - the relocation rows (`ember_rt.c.in:3212-3270`).
- FFI-39c:
  - a keeping parameter takes an `owner="foreign"` object by hand-over;
  - E5060 is given to an `owner="ember"` one, with both lines (`ember_typeck/src/lib.rs:35398-35423`
    rewritten);
  - the trampoline's `deleted` thunk (`cimport.rs:3725`) no longer checks a count, since a handed-over
    object has no other owner (D5);
  - the `owner="ember"` delete panic stays.
- Rules: FFI-17d, FFI-23, FFI-39c, HR-20 (the `on_relocate` half).
- Tests: retire FFI-23 2, HR-20 1, `reload.rs` 1; rewrite FFI-17d 2, FFI-39c 2, `reload.rs:1217` 1.
- Depends on: A6-9; A1-9.

### Slice F3: the checked trampoline

**A6-6. FFI-53 in the runtime and both backends.** L.
- Files:
  - `runtime/ember_rt/templates/ember_rt.{h,c}.in`: the slot table of D1, register, unregister, the
    three stops, generation retire;
  - `ember_codegen_c/src/lib.rs:6786-7070`: record becomes slot; trampoline checks; the `once` mark;
    the release becomes `unregister` with the deferral of verdict 62;
  - `ember_codegen_llvm/src/ffi.rs:40-260, 449`;
  - `std/src/ffi.em:229-245` (`ForeignCallback` holds the number).
- The thread check leaves `EMBER_RT_DEBUG_ACCESS` (every profile). The stop texts name the
  registering line (an `ember_loc` stored in the slot).
- Rules: FFI-53, FFI-21 (trampoline half), FFI-22 (check half), FFI-23 (the token).
- Tests:
  - design R15 6: `run_c_holds_a_tagged_number_not_an_address`,
    `run_c_runs_a_kept_callback_through_the_trampoline`,
    `run_fail_a_creator_callback_called_on_another_thread`, `run_fail_c_calls_a_once_callback_twice`,
    `run_fail_c_runs_a_callback_after_it_was_unregistered`,
    `run_a_kept_callback_that_ends_its_own_token_then_uses_its_state` (Q1);
  - old: FFI-33b `run_fail` rewrite; the FFI-21 respells.
- Depends on: area 04's two-word function value (the block holds `{code, state}`); A9-3 (the AI branch
  edits `ember_codegen_llvm/src/ffi.rs`); the AI branch's HR-42 depth counter in trampolines kept.

### Slice F4: what callbacks and exports may hold

**A6-7. Callback captures and thread rules.** M.
- Files: `ember_typeck/src/lib.rs:52334-52420`, `:35530-35660`, `:35897-35900`.
- Kept callbacks:
  - a capture list (`owns`, `copy of`, `thread_shared`; `shares`, `shares mut`, `weakref` only under
    `threads(creator|main)`);
  - a name with no word is E7004 with FFI-21's message (today E3026/E3063).
- A `threads(any)` callback, borrowed or kept, is checked as a `thread_shared fn()`: E7015 for a
  one-thread owner or a changed real variable, E7008 for a thread-bound value.
- Rules: FFI-21, FFI-22.
- Tests:
  - FFI-21 rewrites 3;
  - design R15 4: `reject_a_kept_callback_using_a_name_with_no_capture_word`,
    `reject_a_callback_c_may_run_on_any_thread_holding_a_shares_owner`,
    `accept_a_callback_c_may_run_on_any_thread_holds_a_thread_shared_owner`,
    `accept_a_creator_thread_callback_holds_a_weakref`;
  - the 10 FFI-22-named function-value tests pass with A5-10.
- Depends on: area 04 (capture words, CLO-*), A5-8, A5-10 (`Send`, `thread_shared fn()`, E7015, E7008).

**A6-14. Exports and thread contracts.** S.
- Files: a check over each export's reachable statics (on area 04's call graph); remove
  `EMBER_OBJ_CHECK_THREAD`'s FFI use (`ember_rt.h.in:1522-1529`) and its test
  (`ember_build/src/lib.rs:2940`).
- `threads=any` (and `creator`): a `static` read is `ThreadShared`, a `static mut` is reached only
  under its lock (E7012 with FFI-33's message).
- Rules: FFI-33, FFI-33a, FFI-33b (every profile, with A6-5 and A6-6).
- Tests: design R15 2 (`accept_an_any_thread_export_changes_a_guarded_static`,
  `reject_an_any_thread_export_changes_a_static_mut`, once respelled per Risk 2).
- Depends on: A5-16, A5-18 (THR-20, E7012), A1-17 (`static mut`), A6-3.

**A6-13. A `thread_shared` class value crosses an export (checker side).** S.
- Files: `ember_typeck/src/lib.rs:21073-21090` (`exported_class`), `:21400-21420`
  (`exported_owner_reachable` gains `owns`/`shares`/`thread_shared`/`weakref`/function values).
- The codegen is A9-9 (both backends: handle copy = atomic +1 with the limit compare of RT-7).
- Rules: BLD-FFI-5a, FFI-31b.
- Tests: BLD-FFI-5a rewrites 2; `ember_driver/tests/staticlib.rs` `a_sync_class_crosses_…` rewrite.
- Depends on: A5-9, A5-11 (`thread_shared` values), area 03 (count header), A9-9, Q3 (area 09's Q3).

### Slice F5: foreign calls in the effect model

**A6-8. What a foreign call may run (FFI-52) and C++ re-entry (FFI-39d).** L.
- Files:
  - parser and typeck: `no_callbacks` in `FFI_CONTRACT_WORDS` (`ember_typeck/src/lib.rs:60360`) and in
    overlays; E5069 when a `callback(…)` parameter is present; FFI-37d's grade rule;
  - area 04's summary builder: the pseudo-callee of D3;
  - the interface file (`ember_build/src/interface.rs`): the package's C-reachable functions;
  - messages (E3020 naming the mark, FFI-52's two texts);
  - remove `ember_analysis/src/long_access_lint.rs`'s FFI-39d case (`:43`, `:127`).
- Rules: FFI-1, FFI-10, FFI-11 (`no_callbacks`), FFI-21 (last sentence), FFI-37d, FFI-39d, FFI-52.
- Tests:
  - design 16: R12 `accept_tick_after_the_loop_over_log_may_run_the_callback_c_keeps`,
    `reject_tick_inside_a_loop_over_log_may_run_the_callback_c_keeps`; R11
    `reject_a_visit_function_that_reads_its_node_after_ending_it`; R15 13 (the three
    `…borrowed_callback…`, the three `accept_a_c_call_inside_a_walk…`,
    `accept_a_c_function_marked_no_callbacks_inside_a_walk`,
    `accept_tick_calls_poll_events_with_no_word_on_its_first_line`,
    `reject_an_unmarked_c_function_inside_the_walk`, `reject_a_c_call_inside_a_walk_a_kept_callback_may_change`,
    `reject_handing_c_the_callback_inside_the_walk`, `reject_no_callbacks_on_a_function_that_takes_a_callback`,
    `reject_tick_called_inside_a_walk_of_log_may_run_the_kept_callback`);
  - old FFI-39d rewrite 1.
- Depends on:
  - area 04 (whole-program effect summaries, function-value targets, interface files);
  - area 02 (the loop, view and `ref mut` rules this feeds: BCK-3, BCK-7, E3020 to E3022, E3060);
  - A6-7 (thread contracts per callback); A6-15 (overrides of C++-derived classes).

**A6-15. Imported C++ methods.** M.
- Files: `ember_driver/src/cimport.rs:3600-3760` (receivers become none; `T&` becomes `ref mut x:`,
  `const T&` becomes `x:`, `T&&` becomes `owns x:`; static members become `static fn`); the overlay
  class line with `@ffi(const)` (`ember_parser/src/decls.rs:693`, `cimport.rs:794-880`).
- A changing method through a read-only name is E3023, and inside a walk of the container it is E3020.
- Rules: FFI-39 (`init`), FFI-40, FFI-40a, FFI-41, FFI-42a.
- Tests: rewrites FFI-40 1, FFI-40a 2, FFI-42a 1; respells FFI-39 9, FFI-40 1, FFI-40a 2.
- Depends on: A1-7, area 02 (E3023, the loop rule), A6-1.

### Slice F6: overlays across packages

**A6-16. An overlay names its header's hash.** S.
- Files: `ember_parser/src/decls.rs:650-688` (`with (header_hash = "…")`); overlay application in
  `cimport.rs` compares it with the importing package's `.embind` hash (`:5593` computes it for C++
  only today; compute it for C too).
- E5010 naming the overlay, the header and both hashes.
- Rules: FFI-30c, FFI-55.
- Tests: new ones; FFI-30c 2 respells (notes).
- Depends on: nothing.

### Slice H: hot reload (after A9-3 merges the AI branch's planner)

**A6-18. Schemas with words; what a reload rewrites and what it may not move.** L.
- Files: `ember_codegen_c/src/image.rs` (AI's planner, after A9-3) or `reload.rs:344-1160`;
  `runtime/ember_rt/templates/ember_rt.h.in:3082-3096` (new kinds).
- The schema hash takes each field type's words (HR-11).
- New kinds:
  - `OWNS` (the old BOX);
  - `SHARES` and `THREAD_SHARED` (the count header of area 03);
  - `WEAKREF`;
  - `VIEW` with its length for `Span`, `MutSpan` and `StrView`;
  - `FN_STATE` (the two-word value);
  - `USING` (the instance address).
- Statics are walked (HR-15).
- An object pinned by A6-9 or a callback block of A6-6 is never moved, and refused naming it
  (HR-15a, HR-20).
- Rules: HR-11, HR-15, HR-15a, HR-20.
- Tests: the 50 `ember_driver/tests/reload.rs` programs respelled; new ones per kind.
- Depends on: A9-3, area 03 (owner representations), A1-6, A1-14, A6-6, A6-9.

**A6-19. Header sizes.** S here (the work is A9-8).
- Rules: HR-12, HR-19.
- Tests: rewrite HR-12 1, HR-19 1.
- Depends on: area 03's header (8 or 12 bytes), A9-8.

**A6-20. The inferred facts in images (HR-44).** L.
- Files:
  - the planner writes each reloadable function's fact record (D7) into the manifest (HR-8);
  - frame records list the functions the rest of a function may still call (HR-3b, `frames.rs`);
  - the runtime's plan compares old and new records for frames below the checkpoint, suspended
    generators and other packages' recorded expectations, and refuses with E9030 and HR-44's report;
  - a `@cached` method's kept result is migrated as removed and added.
- Rules: HR-2a, HR-3b, HR-8, HR-14 (`@cached` row), HR-19 (facts in the bodies tier), HR-41, HR-44.
- Tests: new reload tests (none exist).
- Depends on: area 04 (the summary format, which is the interface file's), A9-3, A6-21 (generators).

**A6-21. Suspended generators across a reload (HR-45).** M.
- Files: the planner (a frame schema per reloadable `gen fn`: layout plus one state per `yield`, with
  what lives across it); runtime (find live frames among enumerable values; resume at the same state;
  refuse naming the `gen fn` and the count).
- Rules: HR-14, HR-19, HR-41, HR-45.
- Depends on: area 05 (CORO-3 frames), A6-18.

**A6-22. `using` values and `@reinit_on_reload` (HR-46, HR-47).** M.
- Files: the planner and runtime.
- Migrate in place when size and alignment match, else refuse. Rewrite the instance word when the
  instance moves. Refuse a schema change of an `Allocator` type while its values live.
- `@reinit_on_reload` is refused where a view, a `sharedref mut` link or an instance address held by
  a static, live instance or registered container may point into the static's value (judged from the
  types).
- Rules: HR-14, HR-41, HR-46, HR-47.
- Depends on: A1-14 (`using`), D3 of area 01, area 03 (kept views), A6-18.

Order inside the job: F1 → F2 → F3 → F4 → F5; F6 any time; H after A9-3 and area 03's header, with
A6-20 after area 04's summaries. Each slice leaves the compiler consistent:
- F1 changes words only;
- F2 removes `ForeignBox` and `Retained.pin` together with the new hand-over;
- F3 swaps the trampoline in both backends at once;
- F4 and F5 add refusals with their tests;
- H changes image format and runtime together.

## 3. Shared decisions

**D1. The callback slot table and its tagged number (FFI-53).** One process-wide table per runtime
copy (FFI-31c).

The table:
- chunks of 4,096 slots, reached through a directory of chunk pointers allocated once, so a slot never
  moves;
- a slot is 64 bytes: `check` (u64: generation << 48 | type tag), `block` (the moved function value
  `{code, state}`), `release` (unregister thunk), `loc` (the registering line), `thread` (u32),
  `state` (u32: running count, used-up bit, unregistered bit).

The number C gets:
- `gen << 48 | slot`, with generations 1..65535, so the top 16 bits are never zero and the number is no
  user-space address on x86-64 or AArch64;
- the type tag is a 48-bit hash of the trampoline's C signature, fixed when compiling.

Per call:
- `slot < published_count`, one compare against an atomic relaxed load;
- `slot.check == (num & ~SLOT_MASK) | TAG`, one 64-bit compare;
- for `threads(creator|main)`: `slot.thread == tls_thread_number`, one compare, now in every profile;
- for `once`: an exchange on the used-up bit;
- for `threads(any)` callbacks that can be unregistered: an atomic increment and decrement of the
  running count. For creator/main callbacks that count is a plain field, since only that thread can
  run or unregister them.

Unregister:
- bump the generation, so a stale number fails the compare; retire the slot at 65535 (one compare per
  registration);
- call `destroys(f)`;
- if this thread is inside the callback (a thread-local stack of running slots), mark the block to be
  ended by the last returning call (verdict 62); otherwise wait for runs on other threads, then end
  and free the block.

Cost against C: a C call through a function pointer plus three compares, about 1.02x to 1.05x per the
design (`RUN3/combined-v4.md` line 549). The atomics of `threads(any)` are unmeasured (Risk 4).

Why: it is the only layout meeting the spec's text, and the type tag folded into the generation word
makes the "type" check free.

**D2. A foreign `owns T` is one word: the foreign pointer.** The destructor is static, `T`'s
`destroys(f)` or the one destructor its contracts name (Q2). A type whose contract says
`threads(creator)` adds a u32 creator (16 bytes with padding). Today's `ForeignBox` is 24 bytes
(pointer, fn pointer, u32), `std/src/ffi.em:165-168`. The end is one direct call to `f` plus the
FFI-36b record removal that exists today. It builds on area 01's D1 (`owns T` is `Box`'s
representation), with the block replaced by the foreign object.

**D3. Foreign calls in the effect summaries: one pseudo-callee per thread class.** For each program
(and recorded per package in its interface file, EFF-4), the compiler collects the C-reachable set,
each member with its thread contract:
- functions handed to `callback(retained|once)`;
- exports and table fields;
- overrides in Ember classes derived from C++;
- functions converted to `extern "C" fn`.

`<C:any>` is the union of the `threads(any)` members. `<C:here>` adds the `creator`/`main` members
whose thread may be the caller's. That is decided from each function's thread context: the roots that
reach it, namely `main`, `thread.spawn` bodies, exports by their policy, and callbacks by theirs.

A foreign call without `no_callbacks` is an edge to the right pseudo-callee. A `callback(borrowed)`
function `B` is added to the foreign calls inside functions reachable from `B`'s body ("whose own call
is still running").

Cost: compile time only. The fixpoint is area 04's SCC solver; C-reachable functions that call C close
cycles it already handles.

**D4. An object given to C is pinned.** The `owned` hand-over of a class object and `into_raw()` set
the header's pinned flag (`ember_obj_pin`, `ember_rt.c.in:3200`) in reloadable builds only. It is one
store, nothing elsewhere. The reload planner refuses to move a pinned object (HR-15a, HR-20). Callback
blocks are pinned by construction (D1).

**D5. FFI-17d's run-time count check becomes a proof.** A keeping C++ parameter takes `owns b:` of an
`owner="foreign"` class, so the hand-over leaves no Ember owner (OWN-1). `ember_cpp_object_adopt`'s
"must be the only handle" check (`ember_rt.c.in:2496`) is then proven and dropped. The
`owner="ember"` delete panic stays, since C++ may still delete what it only borrowed.

**D6. The names an export may not take (E0104).**
- `main`;
- names beginning with `ember_branding::SYMBOL_PREFIX`;
- `ember_module_init`, `ember_module_shutdown`, `ember_module_protocol_v<N>`;
- the C11 standard library's function names, a fixed list in `ember_diag` or `ember_typeck`, the same
  on every target;
- a symbol two Ember functions define.

Compile time only.

**D7. An image's fact record is the interface file's record.** Per function, keyed by symbol:
- what it changes, reads and ends;
- locks it may take;
- view parameters used after a line that may end what they view;
- parent-link facts;
- what it hands to C.

The encoding is the one area 04 chooses for EFF-4, with a hash per function so plan compares hashes
first. One format means a package's interface and a reload image cannot disagree.

## 4. Questions for the owner

**Q1. Does a `Retained[ForeignCallback]` token count as an owner edge to the state of the function C
keeps (`[WK-5]`)?**
- `[WK-5]`: owner edges include "a function value's state, from every function the program can put
  there".
- `[WK-6]`/verdict 99 refuse a loop "opened … only known while running".
- Verdict 62's design test `R15/run_a_kept_callback_that_ends_its_own_token_then_uses_its_state.em`
  (run-pass) stores the token in `game.token`, while the callback owns `me`, a `shares mut` owner of
  that game. So `game → token → callback state → game` is a loop opened only when C calls back.

Why it blocks: A6-6's test and the E3032 check (area 08) give opposite answers.

Proposal: the token is an owner edge, as WK-5's wording reads. The test is rewritten with
`fn leave() weakref me:` and `if me exists as g: g.token = None; g.add_point()`. Verdict 62's run-time
protection is still needed for that form, and for a token kept in a `static mut`.

**Q2. May two contracts name different destructors for one foreign type?**
- `[FFI-36a]`/`[FFI-56]`: an `owns T` value's end "calls the destructor its asserted `destroys(f)`
  names".
- `[FFI-11]`: an `owns T` result "whose end calls `f`" for `returns_owned(destructor=f)`.

When the two differ, or two functions name two destructors for one `T`, the value must carry its
destructor (two words) or the program is refused.

Why it blocks: D2 (one word versus two) and A6-5.

Proposal: one destructor per foreign type. `returns_owned(destructor=g)` must name `T`'s `destroys(f)`
when `T` declares one (E5002 otherwise), and all such contracts of a `T` without `destroys` must agree.

**Q3. May a `cdylib` export the functions its `thread_shared` class handles need?** This is area 09's
Q3, raised by helper AI and still open in 0.9.15. Texts:
- `[FFI-28]`: a cdylib exports "only its `@export` symbols, its `@export_table`s, `ember_module_init`,
  `ember_module_shutdown`, and the marker";
- `[BLD-FFI-5a]` (changed): the header presents a `thread_shared` class value "as a copyable handle
  class … each copy one more owner".

Why it blocks: A6-13 and A9-9 for `cdylib`s.

**Q4. FFI-53 on a 32-bit target.** `[FFI-53]`: the user data is "the callback's slot … in the low 48
bits and the slot's generation in the top 16, which are never all zero, so the number is no user-space
address". A 32-bit target's `void*` has 32 bits, and any non-zero value may be an address. The spec
still names 32-bit targets (`std/src/ffi.em:102`, `[HASH-2]` "on a 32-bit target"). The compiler builds
64-bit targets only today, so this blocks nothing now.

Proposal: say FFI-53's layout is for 64-bit targets, and give a 32-bit target a 16/16 split (slot
below 65536, generation in the high half) without the "no address" promise.

## 5. Risks

1. **UNS-8's note on `unsafe extern` hits nearly every FFI test.** 172 old files, 65 overlay files and
   27 of 29 FFI design programs lack it. For 109 old tests it is the only respell. It is mechanical
   (a script inserting `# SAFETY:` above the block), but it must land with A5-2, or all of FFI turns
   red at once.
2. **The design programs contradict the spec in places; the spec wins, and the programs need
   respelling.**
   - 11 programs write `import c "./kept_events.c"` or `"./wide_sum.c"` with no `unsafe`, which is
     E3100 under FFI-51.
   - 2 write `@export(threads=any)`, which is E3100 under FFI-26. Their expected E7012 text says
     "(`@export(threads=any)`)", where FFI-33's message says "(`threads=any`, line 3)".
   - 3 (`R15/accept_a_borrowed_callback_uses_the_real_variable`,
     `…that_changes_the_list_after_the_walk`, `reject_a_borrowed_callback_that_changes_the_walked_list`)
     change `main`'s `count`/`names` from a `callback(borrowed)` with no `threads(...)` word. FFI-22
     makes that a `threads(any)` `thread_shared fn()`, so E7015, and FFI-22's own message asks for
     `threads(creator)`. The fix is to add `threads(creator)` to their contract. The reject test then
     still gives its E3020.
   - The verdict-62 program is Q1.
3. **FFI-52 rests on effect inference that does not exist yet.** Today's effects are the old effect
   sets (`ember_analysis/src/effects.rs:943-961`). "Whose thread contract allows this thread" needs a
   per-function thread context (D3). Over-approximating it would refuse lines the spec allows, so it
   is part of the rule, not an option. The precision of borrowed-callback nesting is the unknown.
4. **Trampoline cost.** 1.02x to 1.05x is the design's measurement. The `threads(any)` running count
   (two atomics per call) and the release-profile thread compare are unmeasured. Measure on the laptop
   P-core after A6-6 against today's trampoline, quoted as 1.47x.
5. **The AI branch.** `stopped/AI-llvm-backend` rewrites `ember_codegen_c/src/reload.rs` (-1501) and
   `frames.rs` (-281) into `image.rs`. It adds trampoline code (HR-42 depth counter) and `@sync` class
   exports in IR (`ember_codegen_llvm/src/ffi.rs`, 117 lines changed). Conflicts with this area:
   - A6-6 must change both backends' trampolines, and the LLVM one is the branch's version;
   - BLD-FFI-5a's `@sync` exports in IR become `thread_shared` handles (A9-9);
   - every HR item belongs in the branch's planner, so slice H must wait for A9-3, or A9-3 becomes the
     worst merge of the job (area 09).

   Main's LLVM backend has no reload images (the branch notes say `std.hot.checkpoint()` was refused
   under LLVM).
6. **`into_raw()` has two owners.** Area 01 (A1-11) and area 05 (A5-4) both list the built-in. A6-9
   assumes one of them builds the core, and adds only the FFI parts.
7. **HR-20 against FFI-53.** HR-20 says a kept callback's state is never moved because "C holds their
   addresses". Under FFI-53, C holds a tagged number, so the runtime could move a block. I follow
   HR-20's stated behaviour (never moved, refuse when it does not fit). No question raised: the
   behaviour is clear and the cost is a rare refusal.
8. **Codes the new texts use are missing from the registry:** E5013, E5069, E7008, E7012, E7015
   (`ember_diag/src/codes.rs`). E5060's title still names the `Retained` token (`:377`), and E3100's
   rule list must add FFI-26/51/54/56 (area 07).
9. **Not verified.** No check of the statics an any-thread export reaches was found by searching
   `ExportThreadPolicy` users and `FFI-33` references (FFI-33 "partly" may be "not built"). About 250
   tests were classified by script only. The `get` mark may miss a `ForeignBox` method spelled
   otherwise. The 50 Rust reload tests and `cdylib.rs`/`staticlib.rs` were not classified one by one.
