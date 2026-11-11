#$ test: run-pass
#$ rules: DET-3
#$ stdout: 9.0 3.0
## `[DET-3]` — a `@deterministic` function may call only functions that are free of `Nondet`: a
## function that is, a type parameter's function checked per instance, and a call through a `dyn I`
## whose method declares `@deterministic` itself.

pub interface Scale:
    @deterministic
    fn apply(x: f64) -> f64

struct Triple:
    pass

extend Triple implements Scale:
    @deterministic
    fn apply(x: f64) -> f64:
        return x * 3.0

@deterministic
fn plain(x: f64) -> f64:
    return x + 1.0

@deterministic
fn run(s: ref dyn Scale, x: f64) -> f64:
    return s.apply(plain(x))

fn main():
    t = Triple()
    println(run(ref t, 2.0), plain(2.0))
