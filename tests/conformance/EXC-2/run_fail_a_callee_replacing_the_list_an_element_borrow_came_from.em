#$ test: run-fail
#$ rules: EXC-2, EXC-19
#$ profiles: debug, development, release
#$ stdout: before 10
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# ODR-276's question, with the tests: can a callee end the object a borrowed parameter points
# into? Not in safe code (`[RC-5]`, `[EXC-2]`); see the comment on the function below.

class Bag:
    items: Array[int]

    fn init(mut self, owned items: Array[int]):
        self.items = items

## The callee replaces the list its parameter's element came from: as clearing it, a write while
## the caller's read access holds (`[EXC-2]`).
fn read_then_replace(x: ref int, b: Bag) -> int:
    println("before", x)
    b.items = [1]
    return x

fn main():
    b = Bag([10, 20])
    println(read_then_replace(ref b.items[0], b))
