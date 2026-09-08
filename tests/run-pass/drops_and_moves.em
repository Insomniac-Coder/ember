#$ test: run-pass
#$ rules: OWN-2, OWN-3, DRP-2
#$ assert-c: contains("ember_vec_free")

# `[OWN-2]`, `[DRP-2]` — the buffer is freed when `xs` goes out of scope. Fifty
# calls allocate fifty arrays and free fifty arrays; before drops existed this
# leaked every one of them.
fn build(n: i32) -> usize:
    xs: Array[i32] = Array()
    for i in 0..n:
        xs.push(i)
    return xs.len()

# `[OWN-3]` — a move gives the value away, and the new owner drops it. The old
# owner's drop is removed; dropping both would free the same buffer twice.
fn moved() -> usize:
    xs: Array[i32] = Array()
    xs.push(1)
    ys = xs
    return ys.len()

# `[OWN-3]` — moved on one path and not the other. A drop flag decides at run
# time, which is why this is legal rather than an error.
fn conditional(c: bool) -> usize:
    xs: Array[i32] = Array()
    xs.push(1)
    n: usize = 0
    if c:
        ys = xs
        n = ys.len()
    return n

# A struct that owns an array drops the array with it.
struct Holder:
    items: Array[i32]
    tag: i32

fn nested() -> i32:
    inner: Array[i32] = Array()
    inner.push(7)
    h = Holder(inner, 3)
    return h.tag

fn main():
    total: usize = 0
    for i in 0..50:
        total = total + build(20)
    println(total)

    println(moved())
    println(conditional(true))
    println(conditional(false))
    println(nested())
#$ stdout: 1000
#$ 1
#$ 1
#$ 0
#$ 3
