#$ test: run-pass
#$ rules: MONO-3, MONO-6
#$ stdout: 3 1.5
# `[MONO-3]` — `[build] max_instantiations = N` sets a per-generic ceiling on the package's own
# generics; exceeding it is warning `W2220`, naming the generic and its newest instances.

[T]
fn ident(x: T) -> T:     #$ warning[W2220]: `ident` has 2 instances, more than `max_instantiations = 1`
    return x

fn main():
    a: i32 = 3
    println(ident(a), ident(1.5))
