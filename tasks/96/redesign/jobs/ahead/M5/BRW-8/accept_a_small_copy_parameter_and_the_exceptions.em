# Clause BRW-8.3: "a plain parameter of a `Copy` type that holds no `Cell` or `UnsafeCell`, which
# goes in registers when it is no larger than two pointers, except the object of a method whose
# result is a view (`[LT-1]` rule 1)."
#$ test: run-pass
#$ rules: BRW-8, LT-1, FN-1
#$ stdout: 3
#$ stdout: 1
#$ stdout: 7
# `sum` takes a small `Copy` value and means the same as by address. `first`'s object is passed by
# address (a copy would make `ref this.a` a view of the call's own storage, `E3063`), so `r` views
# `p`. A `Copy` struct holding a `Cell` is passed by address, so a change the module variable's own
# name makes across the call shows through `t` (`[FN-1]`).

@derive(Copy)
struct Pair:
    a: int
    b: int
    fn first() -> ref int:
        return ref this.a

fn sum(p: Pair) -> int:
    return p.a + p.b

@derive(Copy)
struct Ticker:
    count: Cell[int]

static mut TICKER: Ticker = Ticker(count=Cell(0))

fn peek(t: Ticker):
    TICKER.count.set(7)
    println(t.count.get())

fn main():
    p = Pair(a=1, b=2)
    println(sum(p))
    r = p.first()
    println(r)
    peek(TICKER)
