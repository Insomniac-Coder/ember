# M8 ahead: tests and spec questions for function values, written before the builders

Written from `docs/spec-source/Ember_v0.9.20_Hardened_14.md` alone (`355cf49f`, tag `spec-h14`) by the
Opus deputy D2 on 2026-11-20, on `deputy` after `main`. M8 is MILESTONES.md's "Function values:
capture words, two words `{code, state}`": A4-9 (capture words and the real variable), A4-10 (two
words, own state, `once`), A4-17 (ERR follow-ups), A9-6 (zero-sized members left out in C, so a
state's size agrees with the analysis), A9-7 (AJ's item type on the one callable type), A5-10
(`thread_shared fn()`) and A6-7 (callback captures and thread rules). `decisions.md` names them in
area 9 Q1 (ODR-278: `if c: f = a else: f = b` gives `f` the callable type both convert to), N53, N97
(the frame's one address in the state word) and N136; each is what the files below follow. Nothing
was built: every expectation is the spec's words and codes. No hardening is cut here.

**Covered elsewhere, not repeated.** In `ahead/M6a`: CLO-17 (5 files), CLO-18 (3, with `copy of`
captures and `[FN-2]`'s `apply(ref mut counter)`), EFF-26 (4), BRW-11's function-value clauses (a
captured real variable as a changing argument; a function value that changes the call's object),
RC-5's stored-function file (`fn() owns players:`, `h.keep = fn(): pass`), DIA-10's `make_counter`
`E3026`. In `ahead/M9`: THR-17 (an unscoped thread's capture list, the loop's own variable, `ref
t`), THR-1 (CLO-16's two `E7015` messages, `on_event` and `Bus`), THR-10, THR-11 and JOB-2 (CLO-7's
`thread.spawn`, `scope.spawn` and `jobs.submit`). In `ahead/M5`: the capture words its BRW-1.7,
BRW-17 and LT-4 files use. In `tests/conformance`: CLO-14 (bounds by a callable type), CLO-19
(`TYP-23/reject_a_lambda_*` carry its two `each_name` messages), GRM-42's `E0100` for `with` and
`E0116` for loop, pattern and lambda names; ERR-4 is credited.

**Not written.** A6-7's FFI-21 and FFI-22 (a kept C callback's capture list and `E7004`, a
`threads(any)` callback as a `thread_shared fn()`): they need a C header and an overlay, the FFI
milestone's harness, as the existing FFI-21 tests have. CLO-3's `extern "C" fn` conversion of a
value with state, which panics (ODR-102): the spec gives no stop text. A9-7's item type itself (its
behaviour is CLO-1's, CLO-3's and CLO-10's files). CLO-13 and CLO-15 are removed: their old tests
retire (§2).

**Conventions** (as the other banks). Each file starts with `# Clause <RULE>.<n>: "<the clause,
quoted>"`; one file per clause and outcome; a refusal's text is the spec's own with the file's line
numbers, else code only; line numbers come from markers; an expectation on its own line claims a
diagnostic anywhere in the file. A case is one `#$ error[...]` or `#$ stdout:` line.

**The probe** (the release `ember.exe` of 2026-11-19, one file at a time; `static mut` read as
`static` in a copy). Its std predates M10's on `main`: the std's own `Arena has no method named
allocate` is set aside. No fault was found in the tests. Everything that fails is not built:
capture words (`E0900`), a lambda using real variables stored as a value (`E3026` today, `[CLO-4]`'s
"provably ends before" is A4-2's keeps facts), local functions with capture words (a parse error
today), `E0116` on a capture list, `E0120`'s and `E0109`'s 0.9.20 texts, a write through a captured
`ref mut` parameter, `Lock` (M9), and `E7012` at a store into a `thread_shared fn()` place. CLO-1's
two accepts check clean today; none was run.

**Totals.** 10 rules, 16 clauses, 23 files (13 accept, 10 reject), 54 cases: 16 error expectations
(9 with the spec's whole text, 7 code only) and 38 output lines.

## 0. Rules covered, and what else each needs

| rule | clauses | files | cases | also needs |
|---|---|---|---|---|
| CLO-1 | 2 | 2 | 6 | none (ODR-278, A9-7) |
| CLO-2 | 3 | 4 | 9 | M4 (`shares`) |
| CLO-3 | 1 | 1 | 4 | M2 (`static mut`) |
| CLO-4 | 1 | 2 | 3 | M6a's summaries (keeps facts, `[EFF-24]`); M4 (`shares mut`) |
| CLO-5 | 2 | 2 | 4 | M7 (`E3032`, the reject); Q2 |
| CLO-6 | 1 | 2 | 7 | none |
| CLO-6a | 1 | 2 | 4 | none |
| CLO-10 | 2 | 4 | 10 | A9-6 (the C state's size) |
| CLO-12 | 1 | 2 | 4 | the parser (a local function with capture words, `[GRM-28]`) |
| CLO-16 | 2 | 2 | 3 | M9 (`Lock`, `[THR-20]`); M6a (the chain, `[EFF-24]`) |

## 1. Rules, clauses and tests

| rule.clause | what it says | test | message |
|---|---|---|---|
| CLO-1.1 | two functions meeting in one place take the callable type both convert to | `accept_two_functions_meeting_in_one_place_take_the_callable_type` (`if`, a second `=`, a list) | — |
| CLO-1.2 | a local holding a stateless function is not captured | `accept_a_local_holding_a_stateless_function_is_not_captured` | — |
| CLO-2.1 | a name with no word is the real variable | `accept_a_name_with_no_word_is_the_real_variable` (the spec's two `on_click` handlers) | — |
| CLO-2.2 | each word means what it means before a value; `owns` hands over, numbers included | `accept_the_words_make_what_they_make_before_a_value`; `reject_owns_hands_the_name_over` (E3040) | none |
| CLO-2.3 | `ref mut` is no capture word (E0120); a name with no word after one with a word (E0116); `owns fn` (E0109) | `reject_what_a_capture_list_may_not_say` | spec (two); none |
| CLO-3.1 | a callable type is one type wherever it is written: parameter, field, local, result, element, `static`; `type Handler` | `accept_a_callable_type_is_one_type_wherever_it_is_written` | — |
| CLO-4.1 | real variables checked where the lambda is written: it provably ends first, or E3026 naming a word | `reject_a_kept_handler_that_changes_a_callers_variable` (the spec's message); `accept_values_that_provably_end_before_what_they_use` (and the message's lines) | spec |
| CLO-5.1 | a `shares` capture is one more owner; the link back is `weakref` | `accept_a_shares_capture_is_one_more_owner_and_weakref_the_way_back` (Q2) | — |
| CLO-5.2 | a loop of owners through a function's state is refused (WK-1) | `reject_a_loop_of_owners_through_a_functions_state` (E3032) | none |
| CLO-6.1 | a function that gives a capture away is a `once fn`: E3013 where it may run twice; a plain `f: once fn` is E3013; a second call E3040; `owns f: once fn` takes both kinds | `reject_a_once_fn_where_a_function_may_run_twice`; `accept_an_owns_once_fn_parameter_takes_both_kinds` | spec (two); none |
| CLO-6a.1 | a `once fn` is called out of a place that owns it; from a class field E3013 | `reject_a_once_fn_called_out_of_a_class_objects_field`; `accept_a_once_fn_called_out_of_a_place_that_owns_it` (a local, `pop`, the message's `mem.replace`) | spec |
| CLO-10.1 | a state in a block of its own carries `Alloc` (E4001 under `@noalloc`); one word or less that the body never writes is inline; real variables are one frame address | `reject_a_state_in_a_block_of_its_own_under_noalloc` (`counter`, `fall`); `accept_a_state_of_one_word_or_none_allocates_nothing` (A9-6's zero-sized capture) | spec (two) |
| CLO-10.2 | stateless is `Copy`; `=` hands a stateful one over; a call through an element copies the two words; replacing ends the state's owns; `==` | `accept_two_words_handed_over_copied_out_and_ended`; `reject_a_stateful_value_is_handed_over_by_eq` (E3040) | none |
| CLO-12.1 | a local function takes capture words and is passed by its name: `owns` hands it over (E3040 after), no word is E3028 | `reject_a_local_function_handed_over_by_its_name`; `accept_a_local_function_with_capture_words_passed_by_name` | none |
| CLO-16.1 | a place whose functions any thread may run says so; such a function holds `thread_shared` values or copies | `accept_functions_that_meet_thread_shared_fn` | — |
| CLO-16.2 | a function meets `thread_shared fn()` only if its chain reaches no unguarded `static mut`: E7012 where it is stored | `reject_a_function_any_thread_may_run_that_reaches_a_main_thread_variable` | spec |

## 2. Old tests the target contradicts (seen, not a sweep)

- `tests/conformance/CLO-1/reject_a_place_holding_one_function_holds_no_other.em` and
  `reject_another_lambda_in_a_place_holding_one.em`: two functions meeting in one place take the
  callable type both convert to (`[CLO-1]`, ODR-278).
- `CLO-10/reject_large_captured_state_in_noalloc_code.em` ("larger than three words"): a state of more
  than one word, or one the body writes, is a block (`[CLO-10]`); `accept_wide_captured_state_is_held_inline.em`
  likewise.
- `GRM-42/reject_*`'s `E0120` text ("it is not a capture word") against `[CLO-2]`'s "without a word the
  function already uses the real `x`, and may change it: `fn(): x += 1`".
- `CLO-6/*`'s `E3013` texts ("may run it more than once", with `owned fn`) against "runs it twice; hand
  over a copy each time instead: ..."; `CLO-6a/*`'s "`self` is a `Holder` object" against the `j.task`
  form.
- CLO-13's and CLO-15's tests (removed rules) retire; CLO-2's, CLO-3's and CLO-4's 0.9.10 forms
  (`owned fn`, a borrowing lambda at a consumed parameter) are respelled or rewritten.

## 3. Spec questions

None is decided here. **Q1 is already logged**: `[GRM-17]`'s "one simple statement" against the
bracketed lambdas with a compound statement (M9's Q4, in `review.md`); the files here avoid such
bodies, with a helper function where one was needed (CLO-5's `count_handlers`).

| # | rule | the question | readings | recommended |
|---|---|---|---|---|
| Q2 | CLO-5, FN-2a, WK-11 | a function holding a link captured `weakref ui` calls `count_handlers(ui)` for `fn count_handlers(w: weakref Ui)`; today's compiler asks for `count_handlers(weakref ui)`, a new link made at the call | (a) a plain parameter reads the caller's link for the call, with no word and no count (`[FN-2a]`: a value given to a parameter with no word is read for the call); (b) the call makes a new link, `weakref ui` | **(a)**. File: `CLO-5/accept_a_shares_capture_is_one_more_owner_and_weakref_the_way_back` |
| Q3 | CLO-6 | the `E3013` message's program: "this function gives `s` away" with the fix `call_twice(fn(): consume(owns copy of s))`. Is the refused function `fn() owns s: consume(owns s)` (a word capture given away), or `fn(): consume(owns s)` (the real `s` handed over from inside the lambda)? | (a) the word capture, which is `[CLO-6]`'s "gives one of its captures away"; (b) the real variable, which by `[CLO-2]` is the maker's and would be `E3013` as a hand-over out of a place the function does not own | **(a)** for the message as written; (b) has its own `E3013` under `[EXP-6]`. File follows (a) |
