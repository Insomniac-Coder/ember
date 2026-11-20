# M11b ahead: tests and spec questions, written before the builders

Written from `docs/spec-source/Ember_v0.9.20_Hardened_14.md` alone (`355cf49f`, tag `spec-h14`) by the
Opus deputy D2 on 2026-11-20, from `main` at `3656833c`. M11's second slice is the gap map's A4-11
(effects and contracts of 0.9.15), A4-12 (what `init` may call), A4-13 (`@cached`), A4-15 (bounds of
a generic function), A4-16 (class rules) and A7-13 (bounds and `dyn` entries). Nothing was built:
every expectation is the spec's words and codes, never the compiler's. The files fail until the
slice is built; a built rule's directory moves to `tests/conformance/<RULE>/`. No hardening is cut.

**Skipped.** Credited (`docs/phase-status.json`, `built_for_0915`): CLS-9, CLS-11. Already tested in
`tests/conformance`, clause for clause, and not written again: EFF-1 (effects through calls, function
values, adapters, drops), EFF-3 (foreign effects), EFF-14 (the inner-loop set), EFF-16 (checks
allowed under `@nopanic(explicit)`, its message), EFF-18 (independent effects), CLS-1's `E2114`,
CLS-8 (`@sync` is gone: `THR-1/reject_sync_is_gone`), CLS-4's `E2110` to `E2113`, CLS-2's codes
(its messages and its reads through callees are new here), CLS-9a's direct changes, TYP-17's written
bounds, TYP-22's `E2050` cases other than the two below, SER-1's round trips. Not written: EFF-11
(the reasons are in `ember inspect --safety`'s side table, not in a diagnostic: a milestone test),
EFF-6a (removed), MOD-9's and BLD-14's bounds halves (a two-package test; the one-package case is
IFC-6's `E2040`), SER-1's generated signatures (the existing round trip writes them).

**Conventions** (M5's and M6a's). Each file starts with `# Clause <RULE>.<n>: "<the clause,
quoted>"`; one file per clause and outcome; a refusal's text is the spec's own, with the file's line
numbers, else code only; line numbers were resolved from markers by a generator. An expectation on
its own line (not trailing code) claims a diagnostic anywhere in the file: used where the spec does
not say where the refusal stands (Q2, Q4). A case is one `#$ error[...]` or `#$ stdout:` line.

**The probe** (`target/release/ember.exe check`, one file at a time; `static mut` read as `static`
in a copy, to see past it). The release `ember.exe` (2026-11-19 14:39) predates M10's std on `main`,
so every file also shows the std's own `Arena has no method named allocate` (E2072): set aside.
One fault was fixed: `/` on two integers (`E2240`, `[TYP-28]`) in EFF-12's accept, now `//`. Every
other failure is a feature not built: contracts on a function type (`@noalloc` above `type`),
`E4010`, `check_disjoint` and `assert_disjoint` as `[DSJ-1]` says, `@cached`, `sharedref mut` (M5's
A2-8), `init`'s calls judged by the methods' bodies (today any call on `this` is a whole use), bound
inference, the viewed-parameter `dyn` entry, E7015's header promise (M9), and two texts (TYP-22's
`Inspect`, CLS-9a's). One finding beside the slice: CLS-3's `super.init(name)` for an `owns`
parameter is E3028 today, where `[CLS-3]` says such an argument takes no word.

**Totals.** 13 rules, 20 clauses, 30 files (12 accept, 18 reject), 56 cases: 29 error expectations
(15 with the spec's whole text, 2 in part, 12 code only) and 27 output lines.

## 0. Rules covered, and what else each needs

| rule | clauses | files | cases | also needs (beyond this slice) |
|---|---|---|---|---|
| EFF-2 | 2 | 3 | 5 | none (verdict 106's contract line) |
| EFF-9 | 1 | 2 | 3 | M6c (`check_disjoint`, `[DSJ-1]`) |
| EFF-12 | 2 | 3 | 4 | M6c (`assert_disjoint`); not written: the `runtime` name's message (`[LT-48]`, a program the spec does not give) |
| EFF-15 | 1 | 1 | 1 | M6c; Q1 |
| CLS-2 | 1 | 3 | 7 | M2 (`static mut`) |
| CLS-12 | 3 | 4 | 13 | M4 (`shares`, `weakref`); not written: the `Atomic`, lock and `guarded by` cases, E7015/E7002 for a `thread_shared` object (M9) |
| EFF-27 | 1 | 2 | 4 | M5's A2-8 (`sharedref mut`) |
| CLS-6 | 1 | 1 | 3 | none (its `@cached` clause; the rest is tested) |
| IFC-6 | 2 | 3 | 5 | none; Q3 |
| TYP-17 | 2 | 2 | 2 | none (its 0.9.20 clauses on inferred bounds; IFC-6's files are its too) |
| TYP-22 | 2 | 2 | 4 | M11's first slice (`dyn` from a struct value); Q4 |
| CLS-4 | 1 | 2 | 2 | M9 (`Send`/`ThreadShared` markers, E7015); Q5 for "Fields cannot be overridden" |
| CLS-9a | 1 | 2 | 3 | none (changes through calls, by the summaries) |

## 1. Rules, clauses and tests

Message column: "spec" = the spec's text; "part" = claimed up to a point (the reason is in the file);
"none" = code only.

| rule.clause | what it says | test | message |
|---|---|---|---|
| EFF-2.1 | a contract above `type Handler = fn()` holds every function stored into such a place (the contract's code); a call through one has no excluded effect | `reject_a_contract_on_a_function_type_holds_every_stored_function` (a field, a list, a local: E4001); `accept_a_call_through_a_place_of_such_a_type_has_no_excluded_effect` | none |
| EFF-2.2 | an interface method's contract binds its implementations (E4010) | `reject_an_implementation_that_breaks_its_interfaces_contract` (at the implementing method, Q6) | none |
| EFF-9.1 | `Aliasing` is only the disjointness compare and a `runtime` test; a count's limit compare is `Arithmetic`; a `weakref` opening is `Stale` | `accept_only_the_disjointness_compare_is_an_aliasing_check` (under `@no_alias_checks` and `@nopanic(explicit)`); `reject_the_disjointness_compare_is_the_aliasing_check` (`check_disjoint`, E4030) | none |
| EFF-12.1 | `@no_alias_checks` permits bounds, arithmetic and stale checks, forbids only the alias checks | `reject_no_alias_checks_forbids_the_run_time_alias_compare` (the `mix` message); `accept_no_alias_checks_permits_bounds_arithmetic_and_stale` (the message's fix) | spec |
| EFF-12.2 | `@static_safe` is E0104 with its message | `reject_static_safe_is_now_no_alias_checks` | spec |
| EFF-15.1 | a verdict counts the listed proofs as applied | `accept_a_verdict_counts_the_listed_proofs_as_applied` (an `assert_disjoint` of one `split_at`'s halves, every profile; Q1) | — |
| CLS-2.1 | `init` calls a method once every field its body reads is set; one that lets `this` out, and a `pub open` class's `virtual` method, only after every field | `reject_the_messages_init_gives` (`on_make`, `load_assets`, `enrol`); `reject_a_method_that_reads_a_field_through_its_callees`; `accept_init_calls_methods_whose_reads_are_set` | spec (three); none |
| CLS-12.1 | kept until a read field changes through any name; `Copy` by value, else a `ref T` view that a clear ends | `accept_a_cached_result_is_kept_until_a_read_field_changes`; `reject_a_kept_view_is_ended_by_a_clear` (E3060) | none |
| CLS-12.2 | E2117 on a struct's method, one with parameters, one that changes anything, a `virtual` one, one reading what other owners may change | `reject_cached_where_a_kept_result_would_be_wrong` (five) | spec (five) |
| CLS-12.3 | a read through a field's object of a class with a weak count is E2117 | `reject_a_cached_read_through_a_link_that_bypasses_the_holder` | part (Q7) |
| EFF-27.1 | a `sharedref mut` or `shares mut` link into what a `@cached` method reads is E2117 at the line that makes it; one to the whole holder is allowed; clears at each changing line | `reject_a_link_into_what_a_cached_method_reads`; `accept_a_link_to_the_whole_holder_and_the_clears_it_needs` | spec |
| CLS-6.2 | kept results end with the object's fields | `accept_kept_cached_results_end_with_the_fields` | — |
| IFC-6.1 | a non-`pub` function with no bounds has them worked out from its body; a method no interface declares is E2072 | `accept_bounds_worked_out_from_the_body` (a method, `+`, `copy of`); `reject_a_method_no_interface_in_scope_declares` | part (Q3) |
| IFC-6.2 | a `pub` function writes its bounds: one E2040 | `reject_a_pub_function_writes_its_bounds` | spec |
| TYP-17.1 | two interfaces in scope declaring the method: E2070 | `reject_two_interfaces_in_scope_declare_the_method` | none |
| TYP-17.2 | a written list that leaves one out: E2040 with the whole line | `reject_a_written_list_writes_every_bound_the_body_needs` (`direct_add`) | spec |
| TYP-22.1 | a method generic over a viewed parameter's type gets one `dyn` entry | `accept_a_method_generic_over_a_viewed_parameter_gets_one_dyn_entry` | — |
| TYP-22.2 | violations are E2050 with the line to write | `reject_the_messages_an_interface_not_usable_as_dyn_gives` (`Inspect`, `Same`) | spec |
| CLS-4.1 | a class others may derive from promises its thread markers in its header, kept by every derived class; a final class's are read from its fields | `reject_a_derived_class_breaks_its_bases_header_promise` (E7015); `accept_a_final_classs_markers_are_read_from_its_fields` | none |
| CLS-9a.1 | nothing changes through a `let` field, through calls included | `reject_a_let_field_changed_through_a_call` (a method's summary, a `ref mut` argument); `accept_a_let_field_read_through_calls` | none |

## 2. Old tests the target contradicts (seen, not a sweep)

- `tests/conformance/ATT-1/reject_attributes_gone_from_ember.em:7` pins "`@static_safe` is gone: write
  `@no_alias_checks`"; `[EFF-12]` gives "`@static_safe` is now `@no_alias_checks`: it forbids run-time
  alias checks only; bounds and overflow checks still run; write `@no_alias_checks`".
- `tests/conformance/EFF-12/accept_static_safe_allows_bounds_and_arithmetic.em` and
  `reject_a_parameter_forcing_a_dynamic_check.em`: by their names, `@static_safe` (now E0104) and
  EFF-6a's removed check; to be respelled and retired with A4-11.
- `tests/conformance/TYP-22/reject_dyn_generic_interface_generic_method.em`: a generic method is not
  always refused now; one generic only over a viewed parameter's type gets one entry (TYP-22.1).

## 3. Spec questions

None is decided here; each has its readings and the one recommended.

| # | rule | the question | readings | recommended |
|---|---|---|---|---|
| Q1 | EFF-15, DSJ-11, DSJ-6, EFF-12 | `left, right = ref mut xs.split_at(1)` then `assert_disjoint(left, right)` inside `@no_alias_checks`: `[DSJ-11]` knows the two apart "with no compare", and `[EFF-15]` computes a verdict "as if every one of them were applied"; `[DSJ-6]` says inside `@no_alias_checks` "each is `E4030`" | (A) accepted: the proof removes the compare from the verdict; (B) E4030: an `assert_disjoint` is always the compare, and its message offers removing it | **(A)**: `[EFF-15]` names `[DSJ-11]` in its list so that a compare it removes changes no verdict; `[DSJ-6]`'s "each" is read as each that compares. File: `EFF-15/accept_a_verdict_counts_the_listed_proofs_as_applied`. P-row: DSJ-6 adds "(one `[DSJ-11]` makes needless is no check)" |
| Q2 | CLS-12, E2117 | where `E2117` stands: every message names "the `@cached` line to remove" by number | (a) on the `@cached` line; (b) on the method's first line; (c) on the line that breaks it (the change, the parameter) | **(a)**, the line the fix acts on, as other messages stand on the line they name. Files claim the messages for the file |
| Q3 | IFC-6, TYP-17 | the E2072 text for a method no interface declares ends "for the type `heaviest` is given" in `[IFC-6]` and "for the type the function is given" in `[TYP-17]` | (a) IFC-6's; (b) TYP-17's | **(a)**: it names the function, as `[DIA-2]` asks. File: `IFC-6/reject_a_method_no_interface_in_scope_declares` claims the part both share. P-row: TYP-17 takes IFC-6's words |
| Q4 | TYP-22, DIA-26 | where `E2050` stands: the existing tests put it on the interface's line and on a `dyn` use | (a) at each `dyn I` the program writes; (b) on the interface's line; (c) on the method that breaks it, once, when the program writes `dyn I` anywhere | **(c)**: one mistake, one message, on the name that caused it (`[DIA-26]`), with the `dyn` use as a secondary span. File claims the messages for the file |
| Q5 | CLS-4 | "Fields cannot be overridden" names no code (W9's question 1, already for Hardened_15): `class Door(Script)` declaring `name`, which `Script` has | (a) `E2110` with a field message; (b) `E2106`'s "already names"; (c) a new code | **(a)**, `E2110`'s row widened to "a method or field that replaces an inherited one", with "`Door` declares `name`, which `Script` already has; a field cannot be replaced: give it another name". No file until it is ruled |
| Q6 | EFF-2, E4010 | where `E4010` stands: the implementing method, or the `implements` line of the type | (a) the implementing method's first line, the interface's contract as a secondary span; (b) the `implements` line | **(a)**, the name that caused it (`[DIA-26]`). File: `EFF-2/reject_an_implementation_that_breaks_its_interfaces_contract` follows (a) |
| Q7 | CLS-12 | the `first_name` message holds "(`[WK-11]`)" and "(`[GRM-44]`)": the spec's cross-references, or words the message prints? | (a) references, not printed; (b) printed | **(a)**: no other message prints a rule's name. File claims the text up to the first of them |
