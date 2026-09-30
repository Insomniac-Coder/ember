#$ test: compile-fail
#$ rules: CLO-14, TYP-17
# D-406 — a bound is an interface or a callable type; anything else was
# dropped without a word.

fn first[T: (int, int)](x: T) -> T:      #$ error[E2020]: a bound is an interface or a callable type
    return x

fn main():
    println(first(1))
