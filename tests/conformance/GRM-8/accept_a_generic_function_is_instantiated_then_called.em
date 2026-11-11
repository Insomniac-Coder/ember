#$ test: run-pass
#$ rules: GRM-8
#$ stdout: 5 20
# `[GRM-8]` — `name[..]` in expression position is an instantiation when `name` denotes a generic
# function or type, an index otherwise: `f[int](x)` instantiates, then calls.

[T]
fn twice(x: T) -> T:
    return x

fn main():
    xs = [10, 20, 30]
    println(twice[int](5), xs[1])
