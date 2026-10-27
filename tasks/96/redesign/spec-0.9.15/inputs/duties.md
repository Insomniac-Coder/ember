# Duties for the spec Ember_v0.9.15_Hardened_1 (read `inputs/brief.md`, `JOB.md` and `inputs/crosswalk.md` first)

## 01 to 08 The writers

Your parts and families are your row of `inputs/crosswalk.md` section 1; the rules and fixes you carry are
in its sections 2 and 3. For each of your parts:
1. read the part in full, and the `V5/rules.md` rules, `V5/vocabulary.md` rows and inventory fixes that
   land in it;
2. rewrite it so that it says exactly what the new design says: every replaced word, rule and example
   brought to the new model; every rule whose subject is gone removed; the new rules added; every
   refusal citing its code and every run-time check its stop text; every example in the new spelling and
   correct under the new rules;
3. keep everything the new design does not touch as it is;
4. run `python -I tasks/spec-0.9.9/tools/check_ids.py` and `check_citations.py` before you finish, and fix
   what they report in your parts (an ID another part cites that you removed: list it in your report);
5. write your report, `OUT/reports/<number>.md`: the rules you changed, removed and added (by ID); the
   mapping from each `V5/rules.md` rule and section 2, 3 and 4 row you carried to its spec IDs; the
   inventory fixes you applied (by C number); the technical points you settled, with reasons; requests for
   other agents' families; anything you could not do.

## 09 Completeness check (phase 2, `OUT/reports/09-completeness.md`)

Read `JOB.md`, `V5/rules.md`, `V5/unsafe-inventory.md` and the crosswalk, then every part (`PARTS/p00` to
`p24`) as the writers left it, and the writers' reports. Check, and list every failure with the part, the
line and why:
1. every rule of `V5/rules.md` (and every row of its sections 2, 3 and 4) is in the spec, saying the same;
2. every verdict 1 to 99 is applied where it applies;
3. every inventory fix C01 to C34 and U1 to U3 is in the spec;
4. nothing of the replaced model is left anywhere in `p00` to `p24`: `Box`, `Shared`, `Weak`, `Cell`,
   `RefCell`, `SyncShared`, `ForeignBox`, `@sync`, `@borrows`, `owned`, `mem.forget`, `upgrade()`,
   `cast ... if`, written effect clauses (`reads`, `changes`), `with` before what a function keeps, the
   run-time exclusivity checks, the run-time index compare; every example in the new spelling;
5. no rule contradicts another (two parts saying different things about one case).
Do not fix anything.

## 10 Consistency check (phase 2, `OUT/reports/10-consistency.md`)

Run `python -I tasks/spec-0.9.9/tools/check_ids.py` and `check_citations.py` (never `check_examples.py`,
which runs the compiler), and report what they find. Then check across all parts `p00` to `p24`: every
rule ID cited exists; every error code cited is in `p17a` with one meaning and agrees with `V5/rules.md`
section 4; every stop text agrees with `V5/rules.md` section 2; section and part cross-references point at
what they name; every example's names are defined. List every failure with the part, the line and why. Do
not fix anything.
