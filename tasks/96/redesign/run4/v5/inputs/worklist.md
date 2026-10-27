# Worklist for run 4, step 4 (the lead's; read after `brief.md` and `JOB.md`)

`V5` = `tasks/96/redesign/run4/v5/`. Line numbers of `rules.md` are v4's (the file before the rule
writer edits it). "Lead" means the lead decided it, under the owner's delegation of codes, stop texts and
messages ("you can handle this"), or as the only safe reading of a verdict.

## 1. What each new verdict changes, and where

| verdict | rules / vocabulary | answers | tests |
|---|---|---|---|
| 53 (record corrected) | R4: E3031's "a call whose body it cannot see counts as using every such parameter" goes; for a call through a function value or a `dyn` method the compiler reads every function the program can put there (whole program); only a C function's code is unseen, and R12's C rules cover it | AN-62 (b) and R11's walk: `this.on_visit(p)` compiles when no function stored in `on_visit` uses its node after ending it | R11's two walk run tests stay accepted; one `reject_`: a function stored in `on_visit` that ends its node and then reads it, refused at the line that stores it |
| 58 (record corrected) | handing an object to C to keep: a parameter declared `owned`, or `into_raw()` (91); the `Retained` token is only for callbacks | AN-78, AN-81 if they say otherwise | none |
| 83 | every capture list: `fn() copy of score`, `fn() owns players`, `fn() thread_shared count`, `fn() weakref game`; the keyword list's `with` row: `with` is only the block (`with g = m.lock():`, `with scope = ...:`, `with log.lock:`) | every NEW block (script done) and prose | script done; `#$` messages and `##` comments |
| 84 | E2085 removed; `owns` hands over every type the same way: after `owns count` the giver cannot use `count` (E3040 if it does); `copy of` copies for every type; R3 | AN-53 (`return fn() owns count -> int:`), AN-54 part 6 (E2085 gone) | E2085's two tests become: `reject_` using `count` after `owns count`; `accept_` of `fn() owns count` and of `fn() copy of score` |
| 85 | R3, E2123: `owns` on the making line wherever the place owns the value, also inside `Option` and a list | AN-41 `left=Some(owns Tree(value=2))`, AN-40 `r.pools.push(owns IntPool(items=[1, 2]))`, AN-66 `shapes=[owns Circle(r=1.0), owns Square(side=2.0)]`, AN-77 `next=Some(owns Link(value=1))` | the R3, R14, R15 tests that make such values; one `reject_` each for `Some(...)` and `push(...)` without `owns` |
| 86 | section 2 below | every answer: no `changes`/`reads`; rows 3 and 7 | the 72 tests of the report's section 1, and every message |
| 87 | R4: E3031 covers `ref mut` parameters (already in v4; keep, cite 87) | none | R4's `reject_a_ref_mut_argument_into_what_the_callee_clears` stays |
| 88 | R4 (overlap), R8 (f): the same whole variable under two names, proved when compiling, is allowed: the compiler compiles that call against its own copy of the function with one name; parts whose positions are not known (`ref mut log[i]` beside `ref mut log[j]`, or beside `log`) stay refused (E3022) | AN-03 if it says otherwise | `run_`: `add_both(ref mut x, ref mut x)` prints 2; `run_`: `rename(ref mut LOG)` with `LOG.clear()` then `s = [String("renamed")]`, `LOG.len()` prints 1; `reject_`: `ref mut xs[i]` beside `ref mut xs[j]` stays |
| 89 | R4's overlap rule: a `thread_shared` object whose fields are guarded by its lock may reach a call through any number of names (a plain argument and inside a function value); E7018 stays for the call inside the lock | AN-75 | R15's four tests that assumed it allowed are right; one `accept_` named for verdict 89 |
| 91 | `into_raw()` only inside `unsafe:` (E3100 outside), built into the compiler (no `mem.forget`); a C function declared `owned` takes an `owns T` by an ordinary hand-over; `ForeignBox[T]` is gone (`owns T`) | AN-78, AN-81 | `reject_`: `into_raw()` outside `unsafe:` (E3100); `accept_`: inside `unsafe:`; `accept_`: `widget_take(first)` to an `owned` parameter |
| 92 | R15 reductions: a float total a task adds to more than once is a reduction too; the last digit may differ from a one-by-one sum, the same every run | AN-71's float part: the refusal goes | `reject_a_float_sum_added_more_than_once_per_task` becomes a `run_` (`[[0.1], [0.2, 0.3]]` prints `0.6`) |
| 93 | R15: a scoped task takes the loop's name `copy of t` (its own copy) or `ref t` (looks at this round's element, no copy); `ref t` refused where the task may outlive what it looks at (`thread.spawn`), the message giving `copy of t`; the task using the loop's name with no word stays refused (E3026 for a scoped task, E7004 for `thread.spawn`), the message giving both words | AN-71, AN-72, AN-74 may use `ref t` where the copy was only for safety | `accept_` with `ref t`; `reject_` `ref t` given to `thread.spawn` |
| 94 | `pub fn spawn[R](owns f: thread_shared fn() -> R) -> JoinHandle[R]:`; E7004's special case for `spawn` goes, verdict 61's rule refuses (E7015); `scope.spawn` keeps `fn()` | AN-76 if it shows `spawn`'s line | the E7004 tests about what a `spawn` function holds now expect E7015 |
| 95 | R8 (c): while a loop walks a list, map, set or string, nothing changes it through its own name, whatever the change (E3020): `xs[0] = 5`, `counts[k] *= 2`, `names[k] = ...` inside the loop are refused; the loop's handle may change the item it is on: `for ref mut x in xs: x = 5`, `for k, ref mut v in counts: v *= 2`; the message gives that line | AN-12 (the map), every answer with such a write | `reject_` for `xs[0] = 5` and for `counts[k] *= 2` inside the loop; `accept_` for the `ref mut` handle forms |
| 96 | section 2 of the rules: the count limit's stop text unchanged; the conformance note that two tests run with `#$ count_bits: 8` | none | R9's and R11's two count-limit tests: `#$ count_bits: 8`, 255 owners (or links), panics "too many owners of a `Player` (255)"; one `accept_`/`run_` that real programs use 32 bits (limit 4,294,967,295) |
| 97 | R3: `e.mesh = owns Mesh(points=[1.0, 2.0])` for a value made later for an `owns` place (E2123 without it); E0109 keeps only `owns` before an existing name (`e.mesh = m` hands `m` over) | AN-40 if it reassigns | `reject_` `e.mesh = Mesh(...)`; `accept_` with `owns`; `accept_` `e.mesh = m` |

Verdicts 1 to 82 stay as recorded, except what 86 and 95 void or reverse (verdict 90 is reversed by 95;
86 voids the spelling of 22's `changes captured`, 29, 30's change list, 40, 63, 64, 69's `changes` part,
75, 76; see `JOB.md` verdict 86).

## 2. Verdict 86 in the rules: no `changes`, no `reads`

What goes:
- R5's clauses `reads a, b`, `changes a, b`, `changes this`, `changes NAME`, `changes T`, `changes
  captured`, `changes captured...`, change lists (`[C, F: fn(Item) changes C]`, `CHANGED`), and every
  rule that checks a written clause against a body. R5's "What a function's and a method's first line
  says" keeps: the parameters with their words, `owns this` (takes the object over), `-> T`, `from
  <name>`; E0118 keeps the order of those that remain (or goes, if one order is all that is left).
- The codes whose only case was a clause: E2105 (a clause that does not match the body), E2107 (`changes
  this` beside other clauses), E2113 (a function value or override that changes more than its type or the
  overridden method says), E2122 (`changes this` on `drop`), E2108's `changes this` half (its `owns this`
  on `init` half stays). Check each in section 3 and 4; a code with any case left keeps that case.
- Function types: `fn()`, `fn(Node)`, `thread_shared fn()`, `owns dyn I` say nothing about changes.

What replaces it (one paragraph in R5, cited everywhere the old clauses were):
- The compiler works out from each function's body (its own lines, the functions it calls, every
  function the program can put in a function value or behind a `dyn` method it calls, and the `drop` of
  every object it may end) what it may change and what it reads, and checks every rule that used the
  clauses with that fact: R8 (c) loops, R8 (g) read-only names, R4's call check (E3031), R10 and R11's
  ends, R12's stored functions, R15's locks and threads, `init` calling a method before every field is
  set (row 7: allowed when the body reads only fields already set), `@cached` (what it reads, from the
  body). Nothing is checked while the program runs.
- Across a package's edge the fact is kept in the package's interface file (R17); a library update that
  changes a fact can make a caller's line stop compiling, and the message names the chain.
- C code cannot be read: R12's C rules (`@ffi(no_callbacks)`, a kept callback counting as possibly run
  by every other foreign call) stay.
- Messages about a change a call makes name the chain of calls: "`tick()` adds to `LOG` (through `note`,
  line 4) while this loop walks `LOG`; call `tick()` after the loop". A message never asks the
  programmer to write a clause.
- Rows 3 and 7: `fn note(msg: String):` (the loop rule still refuses a call that changes the walked
  list), `fn load_assets() -> Array[String]:` (allowed in `init` once `renderer` is set, because its body
  reads only `renderer`).
- Verdict 55's reset: `reset = fn() owns players: players.clear()`; a view opened from a link is still
  not used after `reset()` (E3060), since the compiler sees `reset` may end players.

## 3. Codes and stop texts (lead)

- `into_raw()` outside `unsafe:`: E3100's existing row, message "`into_raw()` stops Ember from freeing the
  widget, so a mistake leaks it or frees it twice; write it inside `unsafe:`".
- `ref t` given to a task that may outlive what it looks at: E3026 (a function value that may outlive a
  place it uses), message "the thread may still run after the loop has moved on and `texts` is gone; give
  it its own copy: `thread.spawn(fn() copy of t: ...)`".
- A `thread_shared` object whose class holds a plain `fn()` (hole 6a's form): E7015 at the line that makes
  the object `thread_shared`, message "`Bus.handlers` holds plain `fn()` values, and a `thread_shared
  Bus` may run them on any thread; write `handlers: Array[thread_shared fn()] = []` in `Bus`".
- `r = ref b` on a `ref mut` name (recheck U4): E3023's row lists it.
- A kind test that can never pass (U5): a new code, checked unused, message "`s` is a `Shape`, which can
  never be a `Path`; remove the test".
- `return ref mut pool.dense[...]` through a read-only `pool` (U6): E3027.
- The trampoline's tagged number (U8): the generation in the top 16 bits, the slot in the low 48 (AN-78's
  0x4080 = 16512 kept); written in R15.
- `callback(once)` called a second time by C (F-14, U9): section 2's stop table gets "C called the
  callback registered on line N a second time; it was registered `callback(once)`, so it may run only
  once".
- E5013 when the compiler cannot see what the C function reads (W6 item 3, area E item 7): name only the
  promotion: "C passes `narrow` as an `int` through `...`; if `sum_wide` reads another type, convert it
  first: `sum_wide(2, wide, narrow as i64)`".
- The count limit with `#$ count_bits: 8`: "too many owners of a `Player` (255)", "too many `weakref`
  links to a `Player` (255)".

## 4. Every point step 3 left open, settled

Owner of each item in brackets: **rules** (agent 01), **answers** (02-04), **tests** (05-08).

Readability failures (`V5/readability.md`, step 3's):
- F-01: the E3060 "open the link again" message reuses `p`; use a new name: `if cam.target exists as q:`
  and `q` for `p` in its block. [rules, answers D, tests T2 T3]
- F-02: settled by 84. [all] F-03: settled by 85. [all] F-04, F-05, F-07, F-08: void by 86.
- F-06: settled by 53's corrected record. [rules, answers D, tests T2]
- F-09: settled by 87. F-11: settled by 89.
- F-10: drop E7007's atomic way out for a task that reads first (it lets the total pass 10); give only the
  lock, as AN-74. [rules, answers E, tests T4]
- F-12: every message gives the exact line to write (`move b.click() below the loop` becomes the line
  and where). [rules, answers, tests]
- F-13: E0113's message names the caller's variable (`a`), not the parameter's (`n`). [rules, answers B,
  tests T1]
- F-14: the stop text in section 3 above. [rules, tests T4]
- F-15: AN-70's prose says the override no longer calls `min`; the code calls `min`: fix the prose.
  [answers D] F-16: rename the local `shared` in `R9/run_two_shares_mut_owners_change_one_counter_with_no_run_time_check.em`. [tests T2]

Step 3's re-check (`V5/safety/recheck.md`):
- Wrong 1 (`R3/accept_the_first_owner_keeps_what_it_could_do`): void by 86. [tests T1]
- Wrong 2: split `R14/reject_as_question_and_as_bang` into two tests. [tests T4]
- Wrong 3-6: add `from std.mem import assert_disjoint` where a message gives `assert_disjoint`, in the
  tests and in the rules' E3021 and W2190 rows. [rules, tests T1 T4]
- Wrong 7: `R14/reject_a_dyn_iterator_whose_steps_change_more_than_its_type_says` goes or changes with
  86 (its refusal was a written clause). [tests T4]
- Refusals without a test: `owns this` on `init` (E2108), `weakref` of a field (E2119), E1010 `@fp(contract)`,
  E1010 `fn drop(mut self)`: add one `reject_` each. [tests T1 T2]
- U1: 85. U2, U3: void by 86. U4, U5, U6, U8, U9: section 3 above. U7: 92. U10: 77 (whole numbers),
  and R9's sentence says "other than by a reduction". [rules, tests T2 T4]
- Hole 3, part 1: a change of an object held in a function value's own state, read by a view reached
  through a `weakref`: with 86 the compiler sees the change from the function's body, so the view does not
  live across the call (E3060 at its later use). Part 2: replacing or ending a function value or an
  `owns dyn I` value ends what its state owns (as verdict 57 for `drop`), so a view of such an object is
  not used after (E3060). Write both into R10/R12; one `reject_` each. [rules, tests T2 T3]
- Hole 6a's form: section 3 above (E7015); one `reject_`, one `accept_` with `thread_shared fn()`. [rules,
  tests T3]
- The two count-limit tests: verdict 96. [tests T2]

The step 3 agents' points (their forms), each settled:
- Rule writer 1 (`ForeignBox.into_raw`): 91. 2: 87. 3: void by 86. 4: `copy of` a struct holding a
  `sharedref mut` field makes one more holder of the same link (79); a struct holding a `ref mut` field is
  a view type (E2116). 5: 89. 6 (the walk's measured saving): not this step.
- Area A (a map's values written in a key loop): 95, refused; the `ref mut v` handle instead.
- Area B 1: void by 86. Area B 2 (does the end of `with g = m.lock():` count as a use after a change for
  E3031?): no; E3031 is about what may be ended or moved under a view, and unlocking ends nothing (53).
- Area C 1, 2: void by 86. 3: 85. 4: E3031 uses the precise fact from the body (`if a is not b:`
  counts), 53.
- Area D 1: 53's corrected record. 2: 84. 3: 85.
- Area E 1: 93. 2: section 3 above (E3026 scoped, E7004 `thread.spawn`, both messages give `copy of t`
  and, for a scoped task, `ref t`). 3: F-10. 4: 94. 5: 85. 6: 89. 7: section 3 above (E5013).
- T-W1 1, 2: void by 86. 3: 85 and 97 (`e.mesh = owns Mesh(...)`; `m = Mesh(...)` then `Entity(mesh=m)`
  writes no `owns`). 4 (the arena address a value keeps): technical, the rule writer settles it safely
  and records it.
- T-W2 1: void by 86. 2: 87. 3 (the turret): E3031 is only for a change that may end or move what the
  parameter views (53); writing a number does neither, so the turret stays accepted. 4: with 53's
  corrected record the unseen call is read; where E3021 and E3031 both fit one line, E3031 is shown (it
  names the case). 5: message fixes (`static fn` gives `return Counter(n=k)`; E0113 gives the call with
  `ref mut`). [rules] 6 (the test runner and package edges): not this step.
- T-W3 1, 3, 4, 8: technical, the rule writer settles them and records them. 2: 95. 5: 81 (a class object
  in its own block survives a push; an inline struct does not, E3060). 6, 7: void by 86. 9: message
  fixes giving complete lines. [rules]
- T-W4 1: F-01. 2: U10. 3: 96. 4: messages name the lines they replace. [rules] 5, 6: void by 86.
- T-W5 1: F-01. 2: 84. 3: void by 86. 4: hole 6a, section 3 above. 5, 6: void by 86. 7: R8 (g)'s
  wording: a message gives only the ways out that compile. [rules]
- T-W6 1: U8. 2: F-14. 3: E5013, section 3 above. 4: 6a. 5, 6: 93. 7: 89. 8: 85. 9: void by 86. 10:
  technical (the rule writer). 11: E7010's second way out runs into E7015; give only the first. [rules]
