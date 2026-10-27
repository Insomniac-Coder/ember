#$ test: run-fail
#$ rules: HEAP-5, RC-5
#$ profiles: debug, development, release
#$ panics: exclusivity violation: overlapping access to Shared[i64]
# ODR-276's question, with the tests: can a callee end the object a borrowed parameter points
# into? Not in safe code (`[RC-5]`, `[EXC-2]`); see the comment on the function below.

class Holder:
    shared: Shared[int]

    fn init(mut self, owned shared: Shared[int]):
        self.shared = shared

## The callee replaces the only owner of the `Shared` its parameter points into: the caller's
## borrow holds the payload's access (`[HEAP-5]`) for the call, so the replacement panics.
fn read_then_replace(x: ref int, h: Holder) -> int:
    h.shared = Shared(9)
    return x

fn main():
    h = Holder(Shared(3))
    println(read_then_replace(h.shared.get(), h), h.shared.get())
