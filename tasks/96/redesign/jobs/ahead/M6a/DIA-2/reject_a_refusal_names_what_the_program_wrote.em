# Clause DIA-2.1: "Every refusal names what was refused in the program's own names (...) `int`, not
# `i64`; (...) at a call, the caller's variable, not the callee's parameter), says why in one line,
# and gives the exact line to write (`[DIA-25]`). A type is named as the program spells it where it
# is written, and otherwise as the declaration it comes from spells it: `[int; 3]` where the program
# writes `[int; 3]`, `i64` where it writes `i64`, `int` for an untyped literal's default and for
# `len()`'s result."
#$ test: compile-fail
#$ rules: DIA-2, DIA-25, FN-2a
# Each expectation pins only the name the clause requires; the rest of each message is not the
# spec's for these programs. The call's line to write names the caller's `a`, not `bump`'s `n`.
# `int` for `len()`'s result and `[int; 3]` follow the text against the lead's earlier ruling: Q10.

fn bump(ref mut n: int):
    n += 1

fn main():
    x: i64 = "a"    #$ error[E2020]: `i64`
    y: [int; 3] = 5    #$ error[E2020]: `[int; 3]`
    xs = [1, 2]
    k = len(xs)
    s: String = k    #$ error[E2020]: `int`
    a = 1
    bump(a)    #$ error[E3028]: `bump(ref mut a)`
