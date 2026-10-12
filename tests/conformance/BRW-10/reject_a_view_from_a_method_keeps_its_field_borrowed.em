#$ test: compile-fail
#$ rules: BRW-10
# `[BRW-10]` — the view `first_a` returns borrows `p.a`, the field its body
# used, for as long as the view is alive.

struct Pair:
    a: Array[int]
    b: Array[int]

    fn first_a(self) -> ref int:
        return self.a[0]

fn main():
    p = Pair([1], [2])
    r = p.first_a()
    p.a.push(4) #$ error[E3021]: `p.a` is borrowed here and mutably borrowed elsewhere
    println(r)
