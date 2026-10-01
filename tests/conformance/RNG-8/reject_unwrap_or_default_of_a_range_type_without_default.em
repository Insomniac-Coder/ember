#$ test: compile-fail
#$ rules: RNG-8, RNG-10, ERR-4
# D-419 — a range type has no default of its own (ODR-093): one made without
# a construction could be outside the range. `unwrap_or_default` of one is
# `E2040`; it was `E0900`, "not implemented yet".

type Level = i32 in 1 ..= 10

fn main():
    a: Option[Level] = None
    println(a.unwrap_or_default())    #$ error[E2040]: `Level` does not implement `Default`, which `unwrap_or_default` needs
