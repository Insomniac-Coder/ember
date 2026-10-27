#$ test: run-pass
#$ rules: R12
#$ stdout: 1
#$ stdout: 2
## Verdict 80, R12 (rules.md lines 749-755): `with owns t` makes `t` the returned function's own state, so
## it outlives `make_counter`, and a change to its own state needs no word: the result type is a plain
## `fn()`. The nearest allowed program to reject_a_returned_function_that_changes_its_makers_local.

struct Tally:
    n: int = 0

fn make_counter() -> fn():
    t = Tally()
    return fn() with owns t:
        t.n += 1
        println(t.n)

fn main():
    next = make_counter()
    next()
    next()
