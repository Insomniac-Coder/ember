#$ test: compile-fail
#$ rules: STD-19, TYP-1
# G8-4 (the owner's design, 0.9.10) — `u256` and `i256` are counts: they add, subtract, compare,
# print and convert with `as`, to and from the other integers. Multiplying, dividing, bit
# operations, their integer methods, a `MIN` or `MAX`, a float conversion, a range of one and
# reading one from text are refused (`parse[i256]` stopped the compiler, and `parse[u256]` stopped at
# `u64.MAX`).

fn main():
    a: i256 = 6
    b: i256 = 3
    c = a * b               #$ error[E2020]: `*` cannot be applied to `i256`
    d = a // b              #$ error[E2020]: `//` cannot be applied to `i256`
    e = a % b               #$ error[E2020]: `%` cannot be applied to `i256`
    f = ~a                  #$ error[E2020]: `~` cannot be applied to `i256`
    g = a << 1              #$ error[E2020]: `<<` cannot be applied to `i256`
    h = a & b               #$ error[E2020]: `&` cannot be applied to `i256`
    i = a.abs()             #$ error[E2020]: `abs` cannot be applied to `i256`
    j = i256.MAX            #$ error[E1010]: `i256` has no constant `MAX`
    k = a as f64            #$ error[E2020]: `i256` cannot be cast to `f64` with `as`
    l = 2.5 as i256         #$ error[E2020]: `f64` cannot be cast to `i256` with `as`
    r = a .. b              #$ error[E2020]: there are no ranges of `i256`
    for x in a .. b:        #$ error[E2020]: there are no ranges of `i256`
        pass
    t = a
    t *= b                  #$ error[E2020]: `*` cannot be applied to `i256`
    u: u256 = 6
    v: u256 = 3
    w = u * v               #$ error[E2020]: `*` cannot be applied to `u256`
    y = u.count_ones()      #$ error[E2020]: `count_ones` cannot be applied to `u256`
    z = u256.MIN            #$ error[E1010]: `u256` has no constant `MIN`
    q = u ..= v             #$ error[E2020]: there are no ranges of `u256`
    m = "5".parse[u256]()  #$ error[E2020]: `parse` does not read `u256`
    n = "5".parse[i256]()  #$ error[E2020]: `parse` does not read `i256`
