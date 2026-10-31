#$ test: compile-fail
#$ rules: TYP-17, MOD-5
# D-403 — a bound naming an interface nothing declares is `E1010`. Both
# compiled and ran: an unknown bound was taken as met by every type, so a
# misspelt `Hash` or an interface never imported (`Hasher` is not a prelude
# name, `[MOD-5]`) silently bounded nothing.

[T: needs Nope]              #$ error[E1010]: cannot find interface `Nope` in this scope
struct S:
    x: T

[T: needs (Hashr, Eq)]     #$ error[E1010]: cannot find interface `Hashr` in this scope
fn g(x: T) -> T:
    return x

fn main():
    s = S(x = 1)
    println(s.x, g(2))
