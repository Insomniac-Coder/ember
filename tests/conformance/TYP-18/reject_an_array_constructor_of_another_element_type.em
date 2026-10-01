#$ test: compile-fail
#$ rules: TYP-18
# `[TYP-18]` — the written type argument fixes what an `Array` constructor
# makes, so an `Array[u8]()` where an `Array[String]` is wanted is a mismatch,
# and `Array` takes one type argument, `String` none (D-456).

fn main():
    x: Array[String] = Array[u8]()     #$ error[E2020]: expected `Array[String]`, found `Array[u8]`
    a: Array[u8] = Array[u8, u8]()     #$ error[E2020]: `Array` takes one type argument, found 2
    s = String[int]()                  #$ error[E2020]: `String` takes no type arguments, found 1
    println(x, a, s)
