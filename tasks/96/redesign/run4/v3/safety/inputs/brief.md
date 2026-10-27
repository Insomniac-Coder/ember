# The shared brief for safety validation (b) (every agent reads this first, all of it)

You are one agent of a team of 13 writing the safety-invariant matrix and the conformance tests for a
programming language's new memory model. Every agent has the same goal; you only have a different duty.
Your duty is in `inputs/duties.md`, in the section with your number. Read this brief, then your duty.
All paths are under `C:\Users\ism19\Code\ember\` unless they start with a drive. `V3` means
`tasks/96/redesign/run4/v3/` and `S` means `tasks/96/redesign/run4/v3/safety/`.

## The goal of the whole rework, the same for every agent

Rework Ember's memory model so that a program written in Ember is, all at once:
1. as fast as C and Rust doing the same work;
2. as memory-safe as Rust or safer, checked when the program is compiled wherever possible;
3. readable like Python: easy to read and to write. That means readability, not a copy of Python. What
   the programmer means stays written in plain words on the thing it describes; the compiler never
   decides what the code does not say.

## This task

This is defensive compiler testing of the owner's own language, authorized by the owner and limited to
this repository (`tasks/96/redesign/SAFETY-VALIDATION-TASK.md`, part (b)). The new model is written down
in `V3/rules.md` (rules R1 to R18, the run-time checks in its section 2, the refusals in section 3, the
error codes in section 4), with its words in `V3/vocabulary.md`, the 84 worked answers in
`V3/area-A.md` .. `V3/area-E.md`, and the owner's binding verdicts 1 to 52 in `V3/inputs/JOB.md`. It is
not built: today's compiler cannot read the new spellings, so **nothing is compiled or run in this
task**. The job is to state, for every rule, what it guarantees and how that would be proved, and to
write the proving tests now, so they are ready when the model is built.

## The matrix (one row per guarantee)

| field | what goes in it |
|---|---|
| id | `R8-3` (the rule, then a number) |
| guarantee | what the rule promises, in one plain sentence |
| enforced by | the part of the compiler that would enforce it: parser, type checker, borrow check (the MIR pass that today checks borrows and regions), code generator, or runtime (`ember_rt`); name today's source file when one does this kind of work now |
| today's tests | today's conformance tests that touch the same guarantee (`tests/conformance/<RULE>/<file>.em`), or "none" |
| missing edge cases | what no test covers yet |
| tests | the file names of the new tests for this row, each `accept_`, `reject_` or `run_` |
| expected result and how it is verified once built | `compile-pass`, `compile-fail` with the code and message, `run-pass` with the output, `run-fail` with the panic; and how it is checked (the runner's expected error, stdout, a sanitizer run for a run-time check) |

The task's priorities come first: ownership and lifetime, mutable aliasing, views kept in lists and
structs, object destruction, reference counting, exclusive changes of class fields, and the contracts at
the C boundary.

## The tests (today's conformance format)

Each test is one file in `V3/conformance/<R#>/` (for example `V3/conformance/R8/`), named by what it
does: `reject_push_while_a_loop_walks_the_list.em`. Today's format, for example
`tests/conformance/BRW-1/reject_mutating_a_viewed_container.em`:

```
#$ test: compile-fail
#$ rules: R8
## R8 (c): a list does not change while a loop goes through it; `push` may move the list.

fn main():
    names: Array[String] = [String("a")]
    for n in names:
        names.push(String("b"))   #$ error[E3020]: `names` may change while this loop goes through it
```

Headers: `#$ test:` (`compile-pass`, `compile-fail`, `run-pass`, `run-fail`), `#$ rules:` (the v3 rules,
`R8`, `R12`), `#$ stdout:` (each printed line), `#$ panics:` (the start of the panic message), and on the
refused line `#$ error[CODE]: message`, with `#$ help:` for the line the message gives. A `## ` comment
under the headers quotes the guarantee. Each program is complete and minimal, every name defined, in the
owner's spellings (`exists as`, `ref mut`, `changes`, `owns`, `shares`, `weakref`, `copy of`, `this`,
type parameters on their own line above a generic declaration). An expected message follows R18: it names
what was refused in the program's own names, says why in one line, and gives the exact line to write;
where `V3/rules.md` or an answer already gives the message, use it word for word.

## Machine limits

- No compiler changes. No git writes at all. Write only the files your duty names, and scratch files
  under `target/redesign5/<your number>/`. Never edit a file another agent wrote.
- Nothing is compiled or run: no `ember`, no C compiler, no programs. Reading today's tests and today's
  compiler source to name the enforcing part is fine.
- Never `cmd`, never `start`, never anything that opens a window. Every claim names its source (a rule
  and its line in `V3/rules.md`, a verdict number, a file).
- If `powershell -NoProfile -Command "Add-Type -AssemblyName System.Windows.Forms;
  [System.Windows.Forms.SystemInformation]::PowerStatus.PowerLineStatus"` prints anything but `Online`,
  stop at once and say so in your form (`on_mains`: false).
- Plain words: the owner reads the results. Anything the rules leave unclear or contradictory goes in
  your form's `for_owner`, with the rule and the line; do not decide it yourself.
- When you finish, fill in the form exactly. `finished` is true only if you did your whole duty. If you
  were stopped for any reason before the end, `finished` is false and `reason` says why.
