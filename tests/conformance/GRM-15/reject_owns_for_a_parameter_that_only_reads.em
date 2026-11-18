#$ test: compile-fail
#$ rules: GRM-15, FN-2a
## `[GRM-15]` — before an argument for a parameter that only reads it is `E3028`, "`show` only reads its
## argument; remove `owns`: `show(xs)`"; an argument for an `owns` parameter without it is `E3028` too.

fn show(xs: Array[int]):
    println(xs.len())

fn take(owns xs: Array[int]):
    println(xs.len())

fn main():
    xs = [1, 2]
    show(owns xs)    #$ error[E3028]: `show` only reads its argument
                     #$ help: remove `owns`: `show(xs)`
    take(xs)    #$ error[E3028]: `take` takes `xs` over
