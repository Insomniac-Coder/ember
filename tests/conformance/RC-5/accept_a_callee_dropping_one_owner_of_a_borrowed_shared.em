#$ test: run-pass
#$ rules: RC-5, HEAP-5
#$ profiles: debug, development, release
#$ stdout: 3 3
#$ stdout: 4
# `[RC-5]`, `[HEAP-5]` — the callee drops a field holding one owner of the `Shared` its parameter
# points into; the caller's local is another, so the value lives for the call (ODR-276's question:
# no callee ends the object a borrowed parameter points into).

class Holder:
    shared: Option[Shared[int]] = None

## The callee drops the field holding one owner of the `Shared` its parameter points into; the
## caller's local is another owner, so the value lives (`[RC-5]`).
fn read_then_drop(x: ref int, h: Holder) -> int:
    h.shared = None
    return x

fn main():
    s = Shared(3)
    h = Holder()
    h.shared = Some(s)
    println(read_then_drop(s.get(), h), s.get())
    s2 = Shared(4)
    println(read_then_drop(s2.get(), Holder()))
