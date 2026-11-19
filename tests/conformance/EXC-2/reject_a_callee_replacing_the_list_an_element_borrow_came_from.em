#$ test: compile-fail
#$ rules: EXC-2, EXC-19, BRW-11
# ODR-276's question, with the tests: can a callee end the object a borrowed parameter points
# into? Not in safe code (`[RC-5]`, `[EXC-2]`): the view of `b.items[0]` the call holds overlaps the
# `ref mut b` it is given beside (`[BRW-11]`), so the call is refused where it is written.

class Bag:
    items: Array[String]

    fn init(owns items: Array[String]):
        this.items = items

## The callee replaces the list its parameter's element came from: as clearing it, a write while
## the caller's read access holds (`[EXC-2]`).
fn read_then_replace(ref x: String, ref mut b: Bag) -> int:
    println("before", x)
    b.items = ["z"]
    return x.len()

fn main():
    b = Bag(["a", "b"])
    println(read_then_replace(ref b.items[0], ref mut b))    #$ error[E3021]: `b.items[0]` is borrowed here and mutably borrowed elsewhere
