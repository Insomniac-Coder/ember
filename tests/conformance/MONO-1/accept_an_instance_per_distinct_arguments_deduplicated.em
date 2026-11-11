#$ test: run-pass
#$ rules: MONO-1, MNG-1, MNG-6
#$ profiles: debug, development, release
#$ assert-c-count: contains("(int32_t x) {") == 1
#$ assert-c-count: contains("(double x) {") == 1
#$ stdout: 3 1.5 3
# `[MONO-1]` — generic code is instantiated per distinct set of type arguments; instances are named
# deterministically and deduplicated: `ident[i32]` called twice is one function, `ident[f64]` another.

[T]
fn ident(x: T) -> T:
    return x

fn main():
    a: i32 = 3
    println(ident(a), ident(1.5), ident(a))
