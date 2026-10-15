#$ test: run-pass
#$ rules: ERR-7, TYP-19
#$ profiles: debug, release, shipping
#$ stdout: 9
#$ 5
#$ 7 abc
#$ 6
# `[ERR-7]` — the prelude gives every type `From` itself (`from(v) = v`), as
# one implementation over a type parameter standing alone: a type with
# conversions of its own keeps them beside it, chosen by the argument's type,
# and a generic body whose `F: From[E]` is given one type for both meets it.

struct A:
    v: int

struct B:
    v: int

extend B implements From[A]:
    fn from(owned value: A) -> B:
        return B(value.v + 1)

fn convert[E, F: From[E]](owned e: E) -> F:
    return F.from(e)

fn main():
    println(B.from(B(9)).v)
    same: B = convert(B(5))
    println(same.v)
    println(i32.from(7), String.from("abc"))
    other: B = convert(A(5))
    println(other.v)
