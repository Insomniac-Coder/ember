#$ test: run-pass
#$ rules: BRW-1, BRW-2, SPN-1
## `[BRW-2]` — "A borrow is live from its creation until the last use of any
## value derived from it. Scope end is irrelevant." The view's last use is
## before the push, so the push is fine.

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    v: Span[i32] = a
    println(v[0])
    a.push(2)
    println(a[1])
#$ stdout: 1
#$ 2
