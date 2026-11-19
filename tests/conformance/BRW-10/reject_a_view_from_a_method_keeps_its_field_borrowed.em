#$ test: compile-fail
#$ rules: BRW-10
# `[BRW-10]` — the view `first_a` returns borrows `p.a`, the field its body
# used, for as long as the view is alive.

struct Pair:
    a: Array[int]
    b: Array[int]

    fn first_a() -> ref int:
        return this.a[0]

fn main():
    p = Pair([1], [2])
    r = p.first_a()
    p.a.push(owns 4)
    println(r)    #$ error[E3060]
