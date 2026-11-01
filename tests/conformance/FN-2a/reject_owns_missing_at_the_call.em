#$ test: compile-fail
#$ rules: FN-2a, OWN-9
# `[FN-2a]` (verdict 147) — a call says `owns` before an argument for an `owns` parameter, whatever
# is given: a name, a copy or a list made in the call. Without it the call is `E3028`, in the
# program's own names, and for a name the help also gives the copy that keeps it.
#$ help: say so at the call: `take(owns xs)`
#$ help: to keep `xs`, give it a copy: `take(owns copy of xs)`
#$ help: say so at the call: `take(owns [1, 2])`

fn take(owns xs: Array[int]):
    println(xs)

fn main():
    xs = [1, 2]
    take(xs)    #$ error[E3028]: `take` takes `xs` over (its first line says `owns xs: Array[int]`)
    take([1, 2])    #$ error[E3028]: `take` takes `[1, 2]` over (its first line says `owns xs: Array[int]`)
