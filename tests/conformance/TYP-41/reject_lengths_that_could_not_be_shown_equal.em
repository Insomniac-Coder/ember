#$ test: compile-fail
#$ rules: TYP-41
## `[TYP-41]` rule 3 — `N // 2 * 2` keeps `//` whole, so it is not shown to be
## `N`: the diagnostic says so, with both normal forms, rather than that the
## two differ.

fn halve[const N: int](xs: [int; N]) -> [int; N]:
    out: [int; N // 2 * 2] = [0; N // 2 * 2]
    return out    #$ error[E2020]: expected `[i64; N]`, found `[i64; 2 * (N // 2)]`, and `N` and `2 * (N // 2)` could not be shown equal

fn main():
    println(len(halve([1, 2])))
