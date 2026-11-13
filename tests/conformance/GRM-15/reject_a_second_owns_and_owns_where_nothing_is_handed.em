#$ test: compile-fail
#$ rules: GRM-15
## `[GRM-15]` — a second `owns` before a value made for an `owns T` place is `E0109`; before a value made
## on the line that is given to a place that is not an `owns T` it ends nothing and is `E0109`; so is
## `owns` before a name whose place already hands the value over with no word.

struct IntPool:
    items: Array[int] = []

interface Pool:
    fn size() -> int

extend IntPool implements Pool:
    fn size() -> int:
        return this.items.len()

class Reg:
    pools: Array[owns dyn Pool] = []

fn f() -> int:
    return 3

fn pass_on(owns xs: Array[int]) -> Array[int]:
    return owns xs    #$ error[E0109]: `return` always hands the value over; remove `owns`: `return xs`

fn main():
    r = Reg()
    r.pools.push(owns owns IntPool(items=[1, 2]))    #$ error[E0109]: one `owns` already says that the pool has a block of its own
    n = 4
    b = owns (n * 2)    #$ error[E0109]: `=` already hands a value made on the line over; remove `owns`: `b = n * 2`
    c = owns f()    #$ error[E0109]: `=` already hands a value made on the line over; remove `owns`: `c = f()`
    xs = owns [1, 2]    #$ error[E0109]: `=` already hands a value made on the line over; remove `owns`: `xs = [1, 2]`
    k = [1]
    m = owns k    #$ error[E0109]: `m = k` already hands `k` over; remove `owns`
    g = owns fn(): println(1)    #$ error[E0109]: `owns` is not written before `fn`
    println(b, c, xs, m, pass_on(owns copy of k).len())
