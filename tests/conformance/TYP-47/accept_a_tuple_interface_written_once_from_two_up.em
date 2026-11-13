#$ test: run-pass
#$ rules: TYP-47, GRM-34
#$ profiles: debug, development, release
#$ stdout: 1 x
#$ 2 true
#$ 3 4
# `[TYP-47]` — a tuple's interfaces are written once from two elements up: `extend[A, B, Ts...]
# (A, B, Ts...) implements I:` is every tuple's of two elements or more whose first two meet `A`'s
# and `B`'s needs, the rest being the pack's types.

interface Ends:
    fn ends() -> String

extend[A: needs Display, B: needs Display, Ts...] (A, B, Ts...) implements Ends:
    fn ends() -> String:
        return f"{this.0} {this.1}"

fn main():
    pair = (1, "x")
    println(pair.ends())
    triple = (2, true, 2.5)
    println(triple.ends())
    many = (3, 4, "five", 6.0, 'c')
    println(many.ends())
