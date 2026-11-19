#$ test: run-pass
#$ rules: RC-5, RC-1
#$ profiles: debug, development, release
#$ stdout: 3 3
#$ stdout: 4
# `[RC-5]` — the callee drops a field holding one owner of the `shares` value its parameter
# points into; the caller's local is another, so the value lives for the call (ODR-276's question:
# no callee ends the object a borrowed parameter points into).

class Holder:
    shared: Option[shares mut int] = None

## The callee drops the field holding one owner of the `shares` value its parameter points into; the
## caller's local is another owner, so the value lives (`[RC-5]`).
fn read_then_drop(ref x: int, ref mut h: Holder) -> int:
    h.shared = None
    return x

fn main():
    s = shares mut 3
    h = Holder()
    h.shared = Some(shares mut s)
    println(read_then_drop(s, ref mut h), s)
    s2 = shares mut 4
    h2 = Holder()
    println(read_then_drop(s2, ref mut h2))
