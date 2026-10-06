#$ test: compile-fail
#$ rules: TYP-5, TYP-4, STD-19
# `[TYP-5]` rule 12 (the owner's ruling of 2026-10-06) — a whole number converts to another
# whole-number type only where the compiler knows every value it can have fits. A parameter may be
# any value of its type; 300 is not a `u8`, and -1 not a `u64`; a name changed anywhere in its
# function is not known; and only whole numbers convert: a float, a `bool` and a `char` do not.

fn any_int(x: int) -> u64:
    return x                        #$ error[E2020]: expected `u64`, found `i64`

fn main():
    n = 300
    b: u8 = n                       #$ error[E2020]: expected `u8`, found `i64`
    k = -1
    u: u64 = k                      #$ error[E2020]: expected `u64`, found `i64`
    j = 5
    v: u64 = j                      #$ error[E2020]: expected `u64`, found `i64`
    j = -3
    f = 2.5
    i: int = f                      #$ error[E2020]: expected `i64`, found `f64`
    t = true
    tb: u8 = t                      #$ error[E2020]: expected `u8`, found `bool`
    println(any_int(1), b, u, v, i, tb, j)
