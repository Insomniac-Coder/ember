#$ test: run-pass
#$ rules: CLO-4
#$ stdout: 1
#$ stdout: 2
## design rules: R12, R3
## Verdicts 80 and 84, R12 (rules.md lines 761-765): "With `owns`, `n` is handed over to the function and
## is its own state, so the returned type is a plain `fn()`: `fn make_counter() -> fn():`". `n` is an
## `int`, handed over as every type is (verdict 84; E2085 is gone). The nearest allowed program to
## reject_a_returned_function_that_changes_its_makers_local.

fn make_counter() -> fn():
    n = 0
    return fn() owns n:
        n += 1
        println(n)

fn main():
    next = make_counter()
    next()
    next()
