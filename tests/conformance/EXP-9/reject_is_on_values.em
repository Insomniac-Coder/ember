#$ test: compile-fail
#$ rules: EXP-9
# `[EXP-9]` — `is` compares identity, which only class handles and
# references have; on anything else it is `E2150`, whose help names `==`.
# `x is None` is the one test of a value. Two references compare only when
# they refer to one type.

@derive(Copy)
struct Pair:
    a: int
    b: int

fn mixed(a: ref i32, b: ref i64) -> bool:
    return a is b    #$ error[E2020]: `is` compares references to one type, found `ref i32` and `ref i64`

fn main():
    println(1 is 1)    #$ error[E2150]: `is` needs class handles, and `an integer` is not one
    p = Pair(1, 2)
    println(p is p)    #$ error[E2150]: `is` needs class handles, and `Pair` is not one
    flag = true
    println(flag is not false)    #$ error[E2150]: `is not` needs class handles, and `bool` is not one
    some: Option[int] = Some(1)
    println(some is Some(1))    #$ error[E2150]: `is` needs class handles, and `Option[i64]` is not one
