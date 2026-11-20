# M12 ahead: tests and spec questions for the C interface, written before the builders

Written from `docs/spec-source/Ember_v0.9.20_Hardened_14.md` alone (`355cf49f`, tag `spec-h14`) by the
Opus deputy D2 on 2026-11-20, on `deputy` after `main`. M12 is MILESTONES.md's "The C interface":
A6-2 (`unsafe import c`), A6-3 (`@unsafe(export)`), A6-6 (the checked trampoline), A6-8 (what a
foreign call may run, C++ re-entry), A6-11 (C function pointers), A6-15 (imported C++ methods), A5-4
(`into_raw()`; decisions.md: "A5-4 is the `ember tcb` listing only", the built-in being A1-11's) and
A5-5 (`ember tcb`'s sections), with A6-7's FFI-21/FFI-22 cases, skipped in the earlier banks for want
of a C harness. Nothing was built: every expectation is the spec's words and codes. No hardening is
cut here.

**The C and C++ beside the tests.** As `tests/conformance/FFI-21/` does, each rule's folder holds its
own `ember.toml`, the headers and C sources its files import and an `overlays/` folder. No
`ember.overlays` is written: by `[FFI-55]` Ember writes the record the first time the package's own
build applies an overlay. `FFI-53/kept.c` is a library with a deliberate bug (its `unkeep` forgets
nothing), so C calls a callback after its token ended and a `once` callback twice; its
`fire_elsewhere` calls the kept callback on a thread of its own (Windows threads or pthreads, as
`FFI-22/ticks.c`). The C++ headers are header-only, with no standard library header (Q6), so a `.hpp`
beside the test is enough. Files that only compile declare what they need by hand in an `unsafe
extern "C":` block that is never linked (`compile-pass` and `compile-fail` run `ember check`).

**Skipped.** Credited (`docs/phase-status.json`, `built_for_0915`): BLD-FFI-1, FFI-25, FFI-26,
FFI-29b, FFI-33b, FFI-55, OWN-12. Already tested in `tests/conformance` with the spec's text, not
repeated: all of FFI-51 (`FFI-51/reject_c_compiled_in_without_unsafe`, the `bridge.c` message;
`accept_an_unsafe_import_without_a_note_is_warned`, `W3012`; `accept_a_header_that_only_declares_needs_no_word`);
`into_raw()` outside `unsafe:` (`OWN-12/reject_into_raw_outside_unsafe`, the message) and FFI-54's
program (`FFI-23/accept_c_keeps_what_is_handed_over_or_given_with_into_raw`,
`FFI-54/accept_into_raw_of_a_foreign_object_gives_it_up`); `E5069` (`FFI-11/reject_no_callbacks_on_a_function_that_takes_one`,
the message); `E5041` under `retained` (`FFI-21/reject_callback_contract_misuse`); FFI-39's `init` and
`super.init` (`FFI-39/accept_an_ember_class_derives_from_a_cpp_class`); FFI-40a's entries
(`FFI-40a/accept_ffi_const_asserts_a_method_only_reads_its_object`,
`reject_an_entry_without_ffi_const_changes_its_object`); TCB-1's grades (`TCB-1/*`, `W5050`,
`E5050`). Not written: FFI-51's header messages (`json.hpp`'s `g_default`, `lib.h`'s `.CRT$XCU`,
`windows.h`'s `uuid.lib`: an MSVC section, a platform header; a second batch if wanted); FFI-53's
callback that ends its own token and then uses its state (gap Q1: a body of several statements, which
`[GRM-17]` refuses in a bracketed lambda, M9's Q4), its running mark, and the 32-bit stop when no slot
is free; `[UNS-1]`'s `spare` message (a raw pointer returned from a function C may call), which the
gap map gives to UNS-1's own area; FFI-37d (no code: a `no_callbacks` fact stays asserted, which
`TCB-2/accept_a_written_grade_stays_asserted` shows for grades); A6-13 and A9-9 (a `thread_shared`
class value crossing an export into C++ needs a C++ host that includes the generated header, a
`staticlib` test, not a `.hpp` beside an `.em`); BLD-FFI-3's `ember tcb` line per library, with
TCB-4's report below.

**Conventions** (as the other banks). Each file starts with `# Clause <RULE>.<n>: "<the clause,
quoted>"`; one file per clause and outcome, several refusals of one clause in one file; a refusal's
text is the spec's own with the file's line numbers, else code only; line numbers come from markers.
Claims match a part of the message (`milestones.rs`, `text.contains`), so a message whose tail names
something the file cannot have is claimed up to it. A case is one `#$ error[...]` line, one `#$
stdout:` line, or a run-fail's stop.

**The probe** (the release `ember.exe` of 2026-11-19, one file at a time, each rule's folder copied to
the scratchpad so nothing is written into the tree). What stops each file today is the not-built
0.9.20 form: a `callback(…)` parameter not written as the C pointer type is `E5002` and the call is
`E2021` for the user data (`ud`), which 0.9.20 removes at the call; capture words are `E0900`; `owns`
before a value is `E0900`; an `owns` overlay parameter given `owns first` is `E3028`; C++ overloads are
not imported (`W5002`), so `E5056` is not reached; a function pointer from C maps to a safe
`Option[extern "C" fn]` and `@ffi(safe) type release_fn` is `E5010`. FFI-53's four files, FFI-22's
borrowed `visit`, FFI-40's accept, FFI-41 and FFI-54 check clean today; FFI-40's `E3023` and FFI-42a's
two `E3020` are reported; none was run. One fault was fixed: FFI-39's override read `e.key`, a field
of a derived C++ struct the importer need not give (a non-standard-layout struct); it prints a number
now.

**Totals.** 13 rules, 22 clauses, 26 files (5 accept, 16 reject, 2 run-pass, 3 run-fail), 38 cases:
28 error expectations (12 with the spec's whole text, 2 in part, 14 code only), 7 output lines and 3
stops, each the spec's text.

## 0. Rules covered, and what else each needs

| rule | clauses | files | cases | also needs |
|---|---|---|---|---|
| FFI-21 | 3 | 4 | 10 | M8/M9 (capture words, `thread_shared fn()`, `E7004`, `E7015`), A6-8 (the hand-over's changes) |
| FFI-22 | 2 | 2 | 3 | M9 (`Send`, `E7015`, `E7008`) |
| FFI-23 | 1 | 1 | 1 | M9 (`[THR-8]`) |
| FFI-39 | 1 | 1 | 1 | FFI-57 (members of one name), A6-15 |
| FFI-39d | 1 | 2 | 1 | A6-8 on the whole-program summaries (area 04), FFI-39c's `owns` hand-over |
| FFI-40 | 2 | 2 | 4 | A1-7 (no receiver), area 02 (`E3023`) |
| FFI-41 | 1 | 1 | 1 | A6-15 |
| FFI-42a | 1 | 1 | 2 | area 02's loop rule, FFI-40a |
| FFI-52 | 2 | 3 | 4 | area 04 (summaries, function-value targets), M9 (`static mut`, `[THR-20]`) for the export |
| FFI-53 | 4 | 4 | 8 | A6-6 (runtime and both backends); the C compiler for `kept.c` |
| FFI-54 | 1 | 1 | 1 | M10 (`Arena`, `using`) |
| FFI-56 | 2 | 3 | 2 | A6-11 |
| TCB-4 | 1 | 1 | 0 | A5-5, and a milestone test in `compiler/ember_driver/tests/milestones.rs` that runs `ember tcb` on the file (the conformance harness has no `#$ command: tcb`, and its claims are diagnostics) |

## 1. Rules, clauses and tests

| rule.clause | what it says | test | message |
|---|---|---|---|
| FFI-21.1 | a function-pointer parameter carries `callback(…)`; `owns` under `retained`/`once`, at the call too; capturing needs `user_data`; `destroys(f)` for `once` too | `reject_what_a_callback_parameter_takes` (`E5002` ×3, `E5041`, `E3028`, `E3040`, `E5040`) | spec for `set_timer`; code for the rest |
| FFI-21.2 | a kept callback's capture list says what it holds; one-thread owners only under `threads(creator)`, or `threads(main)` from a function only the main thread runs | `reject_a_kept_callback_says_what_it_holds` (`E7004`, `E7015`); `accept_the_words_a_kept_callback_holds` (`[FFI-52]`'s program, `copy of`) | spec; spec |
| FFI-21.3 | handing a function to a kept parameter makes its changes now | `reject_handing_c_a_callback_inside_the_walk` (`E3020`) | none |
| FFI-22.1 | a `threads(any)` callback is a `thread_shared fn()`: `E7015` for a one-thread owner, `E7008` for a raw pointer | `reject_a_kept_callback_any_thread_may_run_holds_a_one_thread_value` | spec for the game; code for the pointer |
| FFI-22.2 | it changes the caller's real variables only by their own operations | `reject_a_borrowed_callback_any_thread_may_run_changes_a_list` (`visits.h`, its overlay) | spec |
| FFI-23.1 | a token stays on the thread that registered the callback: `E7008` | `reject_a_token_handed_to_another_thread` | spec |
| FFI-39.1 | an `override fn` matching no virtual method: `E5056`, giving each virtual of the name | `reject_an_override_that_matches_no_virtual_method` (`handlers.hpp`) | spec |
| FFI-39d.1 | a C++ call may run an override C++ may hold: `E3020` in the walk; nothing at run time | `reject_a_cpp_call_may_run_an_override_inside_a_walk` (Q4); `accept_a_cpp_call_after_the_walk` | spec |
| FFI-40.1 | `T&` is `ref mut x: T`, `const T&` plain, `T&&` `owns x: T`, handed over at the call | `accept_reference_parameters_take_their_words` (`shelf.hpp`) | — |
| FFI-40.2 | a changing method through a read-only name: `E3023`; a `T&` result with no lifetime word: unsafe, `E3100` | `reject_a_changing_method_and_a_reference_result_with_no_word` | code; spec |
| FFI-41.1 | a `const` C++ object is read in safe code unless its type has a `mutable` member (or a pointer, or is not `ThreadShared`) | `reject_a_const_object_with_a_mutable_member_read_outside_unsafe` (`stats.hpp`) | spec |
| FFI-42a.1 | inside a walk of a C++ container, a changing method is `E3020`, a `const` one included where it is imported as changing; a read is allowed | `reject_a_changing_method_inside_a_walk_of_a_cpp_container` (`rows.hpp`) | code |
| FFI-52.1 | a foreign call may run every kept callback and export; so may a function that calls it; `E3020` names the mark | `reject_a_foreign_call_may_run_what_c_keeps` (`sqrt`, `poll_events`, `tick`; Q2); `reject_a_foreign_call_may_run_an_export` (Q5) | spec, code, code; part |
| FFI-52.2 | `@ffi(no_callbacks)`: a call of it runs none of them | `accept_a_c_call_marked_no_callbacks_inside_a_walk` | — |
| FFI-53.1 | the user data is a tagged number with bits 48, 55 and 63 set | `accept_c_holds_a_tagged_number_not_an_address` (every profile) | — |
| FFI-53.2 | a call after the token ended stops | `run_fail_c_calls_a_callback_after_its_token_ended` | spec |
| FFI-53.3 | a `once` callback called a second time stops | `run_fail_c_calls_a_once_callback_a_second_time` | spec |
| FFI-53.4 | a `threads(creator)` callback called on another thread stops, in every profile | `run_fail_c_calls_a_creator_callback_on_another_thread` | spec |
| FFI-54.1 | `into_raw()` on a value made `using` another manager: `E2114` | `reject_into_raw_on_a_value_another_manager_made` (Q3) | part |
| FFI-56.1 | a function pointer from C is `unsafe extern "C" fn`, called in `unsafe:` | `reject_a_c_function_pointer_called_outside_unsafe` (`hooks.h`) | spec |
| FFI-56.2 | `@ffi(safe)` on a typedef's entry: scalars and `@layout(c)` only, `E5002` otherwise | `accept_a_c_function_pointer_asserted_safe`; `reject_ffi_safe_on_a_function_pointer_that_takes_a_pointer` | —; code |
| TCB-4.1 | the unsafe and C FFI sections of `ember tcb` | `accept_the_c_interface_facts_ember_tcb_lists` (the entries the report must show are in its comment) | — |

## 2. Old tests the target contradicts

- A `callback(…)` parameter written as the C pointer type, which 0.9.20 makes `E5002` (E5002's row;
  `[FFI-21]`: "The callable type of a `callback(…)` parameter is `fn(P…) -> R`"), in 8 files:
  `FFI-11/reject_no_callbacks_on_a_function_that_takes_one` (its `E5069` text stands),
  `FFI-21/accept_a_borrowed_callback_takes_a_lambda`, `accept_native_callback_adapters`,
  `accept_user_data_named_by_its_place`, `reject_callback_contract_misuse`,
  `reject_user_data_places_and_nullable_callbacks`, `reject_what_a_callback_captures`,
  `FFI-9/accept_borrowed_aggregate_export_by_value`.
- 9 FFI-21 files write callbacks in the 0.9.15 form `owned fn(…)`, and the kept ones use names with
  no capture word (`accept_callbacks_c_keeps_past_the_call`: `owns owned fn(…)` using `first`,
  `scale`, `tag`): 0.9.20 writes `owns fn() owns x: …`, and a name with no word is `E7004`
  (`[FFI-21]`).
- `FFI-22/run_fail_a_creator_callback_called_on_another_thread`: its stop text ("a callback whose
  contract says `threads(creator)` is called on thread") is not `[FFI-53]`'s ("C called the callback
  registered on line 16 on another thread, but its contract says `threads(creator)`"), and its comment
  says the debug profile where the check is in every profile.
- `FFI-39d/run_fail_a_conflicting_reentrant_access_panics`: "Nothing is checked while the program
  runs" (`[FFI-39d]`); the refusal is `E3020` when compiling (the gap map: remove).
- `FFI-40/accept_methods_and_references` and `reject_a_reference_result_borrows_the_receiver`: a `T&`
  result with no lifetime word is a `ref mut` borrowing the receiver there; 0.9.20 keeps the method
  unsafe to call, its result a `*mut T` (`[FFI-40]`); the comments' `mut self` and `owned T` are the
  0.9.15 forms.
- `FFI-42a/reject_a_method_that_could_invalidate_the_loop_s_iterators`: `println(bag.size())` inside
  the walk is claimed `E3021`, and `size` is `@ffi(const)` in its overlay, a read, which `[FFI-42a]`
  allows (only a method imported as changing is refused).
- `FFI-39c/accept_cpp_keeps_an_object_handed_over`: `cpp.add_widget(button)` gives an `owns` parameter
  its argument with no word (`E3028`; `[FFI-39c]` writes `panel.add_child(owns b)`).
- `virtuals=[…]` in 15 files (`FFI-39/reject_virtuals_names_a_method_that_is_not_virtual` and the
  overlays of DIA-5, FFI-17c, FFI-17d, FFI-39, FFI-39c, FFI-39d, FFI-39e): gone (the spec's list of
  removals: "`@ffi(trampoline, virtuals=[…])` and its unsupported-construct row are gone (ODR-221, the
  owner)").

## 3. Spec questions

None is decided here.

| # | rule | the question | readings | recommended |
|---|---|---|---|---|
| Q1 | FFI-52, LNT-1 | `[FFI-52]`'s program writes `token = setup(shares mut game)` and never reads `token`, which is `L1001` ("a local never read on any path (names starting `_` are exempt)"), so the program as written warns | (a) `L1001` stands and the example writes `_token`; (b) a binding kept for what its end does (a `Retained` token, a guard) is exempt | **(a)**: the lint reads no types, and `_token` says the binding is kept for its end. File: `FFI-21/accept_the_words_a_kept_callback_holds` writes `_token` |
| Q2 | FFI-52, FFI-53 | the `E3020` message's "the callback C keeps from `register_cb` (line 14)": in the section's program line 14 is `register_cb`'s declaration, and the message's own program is not shown | (a) the line of the call that registered the callback, as `[FFI-53]`'s stops and `[FFI-23]`'s `E7008` say "the callback registered on line N"; (b) the declaration of the registering function | **(a)**, one meaning for "registered on line N" in every message. File: `FFI-52/reject_a_foreign_call_may_run_what_c_keeps` |
| Q3 | FFI-54 | `E2114`'s message names `w`, made in `frame`, and its help writes `second = owns Widget(id=2)`, another name; in FFI-54's own program `Widget` is an `extern type`, for which `owns Widget(…)` made in Ember is itself `E2114` (`[FFI-36a]`) | (a) the help names the value the message names, with its own type: "make it there: `w = owns Point(x=2)`"; (b) as written | **(a)**. The file claims the message up to the help |
| Q4 | FFI-39d, FFI-52 | "an override of `Button` whose objects C++ may hold (line 6)": which line, and what makes a class's objects ones C++ may hold | (a) the class's declaration line; a class qualifies when the program gives C++ an object of it that C++ may keep (an `owns` parameter, `into_raw()`), and then every object of it counts ("a `Button`'s `codes`"); (b) the line where an object is given to C++, for that object only; (c) every class derived from a C++ class | **(a)**, which the message's "a `Button`'s" says. File: `FFI-39d/reject_a_cpp_call_may_run_an_override_inside_a_walk` (C++ holds one `Button`; the walk is of another's) |
| Q5 | FFI-52 | "`log_line` (exported, line 5), which adds to `LOG` (line 7)": the line of the `@unsafe(export(…))` attribute or of the `fn` | (a) the attribute's, where the word `export` is (line 6 is then the `fn`, line 7 its first statement, as the numbers fit); (b) the `fn` line, as a function is named elsewhere | **(a)**, which fits the message's own numbers. File: `FFI-52/reject_a_foreign_call_may_run_an_export` |
| Q6 | FFI-51, FFI-42a | with MSVC, the standard library's headers give the linker an order (`#pragma comment(lib, …)`), so an `import cpp` of a header that includes `<vector>` is `unsafe import cpp` there and plain elsewhere; `FFI-42a/containers.hpp` (`#include <vector>`) is imported without `unsafe`. Is a program's import word to differ by C compiler? | (a) yes, as `[FFI-51]` says it judges each header by what it holds with the compiler used (the `windows.h` example); the existing tests then need `unsafe` with MSVC; (b) the word is decided by the most demanding supported compiler, so one spelling builds everywhere | **(b)**: `[FFI-53]`'s reason for stopping on every target, "so that a program's text means one thing wherever it is built"; under it the FFI-42a files take `unsafe import cpp`. No file here includes a standard header |
