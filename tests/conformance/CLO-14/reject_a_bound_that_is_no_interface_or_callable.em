#$ test: compile-fail
#$ rules: CLO-14, TYP-17
# D-406 — a bound is an interface or a callable type; anything else was
# dropped without a word.

[T: needs ((int, int))]      #$ error[E2020]: a bound is an interface or a callable type
fn first(x: T) -> T:
    return x

fn main():
    println(first(1))
