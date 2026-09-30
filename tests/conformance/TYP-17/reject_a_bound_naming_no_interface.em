#$ test: compile-fail
#$ rules: TYP-17, MOD-5
# D-403 — a bound naming an interface nothing declares is `E1010`. Both
# compiled and ran: an unknown bound was taken as met by every type, so a
# misspelt `Hash` or an interface never imported (`Hasher` is not a prelude
# name, `[MOD-5]`) silently bounded nothing.

struct S[T: Nope]:              #$ error[E1010]: cannot find interface `Nope` in this scope
    x: T

fn g[T: Hashr + Eq](x: T) -> T:     #$ error[E1010]: cannot find interface `Hashr` in this scope
    return x

fn main():
    s = S(x = 1)
    println(s.x, g(2))
