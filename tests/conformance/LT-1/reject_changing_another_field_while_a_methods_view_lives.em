#$ test: compile-fail
#$ rules: LT-1, BRW-10, BRW-1
# `[LT-1]` — a method's result points into its object, and pointing into an argument means pointing
# into it and everything it owns: any other change of the object while the view is used is refused
# (`[BRW-1]`), whatever fields the method's body uses (`[BRW-10]` narrows only what the call changes).

struct Pair:
    a: Array[int]
    b: Array[int]

    fn first_a() -> ref int:
        return ref this.a[0]

fn main():
    p = Pair([1], [2])
    r = p.first_a()
    p.b[0] = 3    #$ error[E3021]
    println(r, p.b)
