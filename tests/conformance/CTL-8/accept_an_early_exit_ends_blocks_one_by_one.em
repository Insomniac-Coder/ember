#$ test: run-pass
#$ rules: CTL-8, CTL-7, OWN-2
#$ profiles: debug, release
#$ stdout: drop 1
#$ defer f
#$ 2
#$ defer g
#$ drop 11
#$ drop 10
#$ 1
#$ defer h 0
#$ drop 21
#$ drop 20
#$ defer h 1
#$ defer inner
#$ drop 31
#$ defer outer
#$ drop 30
# `[CTL-8]` (D-672) — "On every exit from a block, its defer blocks run first and then its locals are
# dropped": a `return` or `break` leaving several blocks ends them one by one, the innermost first.
# It used to run every pending `defer` first, before an inner block's locals. A local declared after
# a `defer` in the same block still drops after that `defer` runs.

struct Marker:
    n: int

extend Marker:
    fn drop(mut self):
        println("drop", self.n)

fn f(early: bool) -> int:
    defer:
        println("defer f")
    if true:
        _m = Marker(1)
        if early:
            return 2
    return 0

fn g() -> int:
    _a = Marker(10)
    defer:
        println("defer g")
    _b = Marker(11)
    if true:
        return 1
    return 0

fn h():
    for i in 0..3:
        defer:
            println("defer h", i)
        if i == 1:
            _m = Marker(20)
            break
        _n = Marker(21)

fn k():
    defer:
        println("defer outer")
    _o = Marker(30)
    if true:
        defer:
            println("defer inner")
        _i = Marker(31)
        if true:
            return

fn main():
    println(f(true))
    println(g())
    h()
    k()
