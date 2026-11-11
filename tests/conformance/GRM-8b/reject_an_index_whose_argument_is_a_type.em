#$ test: compile-fail
#$ rules: GRM-8b
# `[GRM-8b]` — an index whose argument is a type is `E2172`.

fn main():
    xs: Array[i32] = Array[i32]()
    n: i32 = xs[ref i32]            #$ error[E2172]: cannot index with a type
    println(n)
