#$ test: run-pass
#$ rules: TYP-41
#$ stdout: 1 2 9 3
#$ stdout: 6
## `[TYP-41]` rule 2 — in generic code a const argument is compared by its
## normal form: `N + 1` and `1 + N` are one, and `2 * N` and `N + N`.

fn grow[const N: int](xs: [int; N]) -> [int; N + 1]:
    out: [int; 1 + N] = [0; N + 1]
    for i in 0..N:
        out[i] = xs[i]
    out[N] = 9
    return out

fn double[const N: int](xs: [int; N]) -> [int; 2 * N]:
    out: [int; N + N] = [0; 2 * N]
    return out

fn main():
    g = grow([1, 2])
    println(g[0], g[1], g[2], len(g))
    println(len(double([1, 2, 3])))
