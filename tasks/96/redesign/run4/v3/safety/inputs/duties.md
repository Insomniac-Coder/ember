# Duties for safety validation (b) (read `inputs/brief.md` first)

`V3` = `tasks/96/redesign/run4/v3/`, `S` = `tasks/96/redesign/run4/v3/safety/`.

## W1-W6 Writers

| # | rules | the guarantees to cover first |
|---|---|---|
| W1 | R1, R2, R3, R13 | one owner and hand-over; a name for a part is a read-only view; words before values, in types and in lists of names (`owns`, `shares`, `shares mut`, `copy of`, `using` and arenas); opening with `exists as` |
| W2 | R4, R5, R6, R17 | parameters and calls (`ref mut` at the call, proving arguments apart, verdict 32's word in front of the name, verdict 42's `const ref mut`); what a first line says (`changes`, `reads`, `changes this`, checked both ways); views a call gives; inferred facts at a package's edge |
| W3 | R7, R8 | views kept in structs and lists (`@view`, `from static`); views and changes checked when compiling: the list rule for every loop and generator, `ref mut` keeping out other reads and writes, a read-only name's promise, lost changes on temporaries |
| W4 | R9, R10, R11 | several owners and shared links (counts, `sharedref mut`); keeping an object alive under a view; `weakref` links (counted, tested on read), parent links set by the program, the walk that stops at the first deleted parent and the deleting walk of verdict 28 |
| W5 | R12 | function values: where captured variables live, `changes` in function types and lambdas (`=>` too), `captured` / `captured...`, `changelist`, lock changes named (verdicts 29, 44), template parameters for functions called right away (verdict 38), stored functions run from a loop, `once fn`, a function that removes itself |
| W6 | R14, R15, R16, and the C side | `dyn` (with `changes`, verdict 46), `is ... as` and `cast ... as`; threads (scoped tasks, `spawn` counting as making the changes, `thread_shared`, `guarded by`, a lock taken twice refused (verdict 43), spawn and join under a lock (verdict 52), channels (verdicts 36, 37)); `unsafe`; C callbacks through the checked trampoline, `@ffi(no_callbacks)` (verdicts 31, 39) |

Read all of `V3/rules.md` (the rules lean on one another), your rules' entries in `V3/vocabulary.md`, the
answers in `V3/area-*.md` that use your rules (search for `R<k>`), and today's tests under
`tests/conformance/` that touch the same guarantees.

Write `S/matrix-W<n>.md` (your matrix rows, in the brief's form, every guarantee of your rules, the
refusals of section 3 and the run-time checks of section 2 that belong to them included) and your tests
in `V3/conformance/R<k>/`. For every refusal at least one `reject_` test and one `accept_` test of the
nearest program that is allowed; for every run-time check a `run_` test with its output or panic. Where an
answer in `V3/area-*.md` already has the program, start from it and name the answer in the `## ` comment.

## V1-V6 Checkers (one per writer, starting when that writer is done)

Inputs: the brief, `V3/rules.md`, `V3/vocabulary.md`, `V3/inputs/JOB.md`, your writer's
`S/matrix-W<n>.md` and its tests.

Check every test and every matrix row against the rules, and try to show each one wrong:
- the expected result: would the rules really accept, refuse, print or panic as the test says? Name the
  rule and line that settles it; when the rules do not settle it, mark it **unsettled**, not right;
- the expected code and message: is it the code `V3/rules.md` section 4 gives this refusal, and does the
  message follow R18 (the program's own names, why in one line, the exact line to write, a line that
  compiles in that program)?
- the test itself: does it test what its name and its `## ` comment say; is it complete and minimal,
  every name defined, in the owner's spellings?
- the row: is the enforcing part plausible, are today's related tests really related, and which edge
  case of the guarantee has no test?

Write `S/check-V<n>.md`: one line per test and per row, **right**, **wrong** (why, and the fix) or
**unsettled** (the rule text that leaves it open). Do not edit the writer's files.

## C1 Completeness checker (after all six writers are done)

Inputs: the brief, `V3/rules.md`, the six `S/matrix-W<n>.md` and `V3/conformance/`.

Find what no writer covered: every refusal in `V3/rules.md` section 3 and every error code in section 4
with at least one `reject_` test; every run-time check in section 2 with a `run_` test; every one of the
task's priorities (ownership and lifetime, mutable aliasing, views kept in lists and structs, destruction,
reference counting, exclusive changes of class fields, contracts at the C boundary); and the places where
two rules meet (a `weakref` opened inside a loop over the list holding its target; a function value
stored in a `shares mut` object changed by a thread; a C callback that removes itself). Write
`S/completeness.md`: each gap with the rule, why it matters, and the test to add (its name and the
program). Do not edit other agents' files.
