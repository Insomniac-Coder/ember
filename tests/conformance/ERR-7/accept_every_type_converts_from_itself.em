#$ test: run-pass
#$ rules: ERR-7, TYP-19
#$ profiles: debug, development, release
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
    static fn from(owns value: A) -> B:
        return B(value.v + 1)

[E, F: needs From[E]]
fn convert(owns e: E) -> F:
    return F.from(owns e)

fn main():
    println(B.from(owns B(9)).v)
    same: B = convert(owns B(5))
    println(same.v)
    println(i32.from(owns 7), String.from("abc"))
    other: B = convert(owns A(5))
    println(other.v)
