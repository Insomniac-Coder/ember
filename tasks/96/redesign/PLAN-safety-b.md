# Safety validation (b): the invariant matrix and tests for the v3 rules (PLAN, for the owner's approval)

## Goal

For every rule of the new memory model (`run4/v3/rules.md`, R1 to R18), write down what it guarantees,
which part of the compiler would enforce it, which of today's tests already touch it, the missing edge
cases, and the tests that prove it, written in today's conformance format. On paper: the model is not
built, so nothing is compiled or run; every expected result is checked against the rules by an
independent reviewer. This is part (b) of `SAFETY-VALIDATION-TASK.md`; part (a), the pipeline on today's
compiler, is planned after it.

## What each test looks like

Today's format (`tests/conformance/BRW-1/accept_the_view_ends_before_the_write.em`):
`#$ test: run-pass | compile-fail | run-fail`, `#$ rules:`, `#$ stdout:`, `#$ panics:`, plus the expected
error code and message for a refusal. Kept under `run4/v3/conformance/<R#>/`, named by what they do
(`reject_push_while_a_loop_walks_the_list.em`), until the model is built.

Each matrix row: the guarantee; the enforcing part (parser, type checker, borrow check, code generator,
runtime); today's related tests (by their `#$ rules:` and content); missing edge cases; the positive,
negative and run-time tests (file names); the expected result and how it will be verified once built.

## Jobs

Phase 1, writers (one per group of rules that belong together, the task's priorities first):

| # | rules | why its own job | effort |
|---|---|---|---|
| W1 | R1, R2, R3, R13: one owner, hand-over, words, `exists as` | ownership and lifetime | high |
| W2 | R4, R5, R6, R17: parameters, calls, first lines, views a call gives, package edge | mutable aliasing at calls, effect summaries | high |
| W3 | R7, R8: kept views, views and changes (the list rule, loops, `ref mut`) | container-held views, the core of the model | xhigh |
| W4 | R9, R10, R11: shared owners, keep-alive, `weakref` and parent links, the deleting walk | reference counting, destruction | high |
| W5 | R12: function values (captured, `changes` in types, locks, `changelist`, templates) | 150 lines of rules, its own world | xhigh |
| W6 | R14, R15, R16 and the C side: `dyn`, downcasts, threads, locks, `unsafe`, C callbacks and `@ffi(no_callbacks)` | threads and FFI ownership contracts | xhigh |

Phase 2, checkers (each starts as soon as its writer finishes):

| # | job | effort |
|---|---|---|
| V1-V6 | one per writer: try to prove each test's expected result wrong against the rules (accept that should refuse, wrong code or message, a message that breaks R18, a test that does not test what its name says); default to "wrong" when the rules do not settle it | high |
| C1 | completeness: what falls between the groups (a rule's interaction with another, the task's priority list, every refusal in the rules' section 3 and every runtime check in section 2 has a test) | xhigh |

13 agents, Opus 5.5 (no Fable); worst case 26 with one retry each.

Phase 3, the lead: merge into one matrix, apply the checkers' confirmed fixes, list for the owner any place
where the rules themselves turned out unclear or contradictory (questions, one at a time).

## Safeguards (as in step 2)

The same shared brief (the goal word for word); every form has `finished`; a not-finished agent is
retried once, then the run halts; if a safety filter stops an agent, the run stops and the lead reports,
no rewording. No compiler changes, no git writes, nothing compiled or run (the tests use the new
spellings, which today's compiler cannot read); scratch under `target/redesign5/`. Power checked.

## Outputs

`run4/v3/safety/matrix.md` (the matrix), `run4/v3/conformance/` (the tests), `run4/v3/safety/checks.md`
(the checkers' results), and a short report for the owner: rules covered, tests written by kind, what the
checkers found and fixed, and open questions.

## Not in this plan

Running anything: the run-time tests state their expected output and are verified when the model is
built. Measuring (owed: W2's parent-link counts, verdict 47's arena pointer) and part (a) come after,
each with its own plan.
