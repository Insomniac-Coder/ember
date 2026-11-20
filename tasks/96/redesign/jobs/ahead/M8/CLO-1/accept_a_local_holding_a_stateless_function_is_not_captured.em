# Clause CLO-1.2: "A local holding a function that keeps no state is not captured: a lambda that uses
# it keeps nothing for it (`[CLO-2]`), since giving it the same function again changes nothing."
#$ test: run-pass
#$ rules: CLO-1, CLO-2, CLO-4
#$ stdout: 8
# `g` holds `seven`, which keeps no state, so the lambda keeps nothing for `g` and may outlive
# `make` (no `E3026`).

fn seven() -> int:
    return 7

fn make() -> fn() -> int:
    g = seven
    return fn() => g() + 1

fn main():
    h = make()
    println(h())
