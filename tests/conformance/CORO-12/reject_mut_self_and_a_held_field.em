#$ test: compile-fail
#$ rules: CORO-12
#$ profiles: debug
# `[CORO-12]` — a class generator method takes `self`: `mut self` is `E2229`, and so is a
# long-term access to a field of the object live across a `yield`.

class Bag:
    items: Array[int] = []

    gen fn drain(mut self) -> Generator[int]:    #$ error[E2229]: takes `self`, not `mut self`
        yield 1

    gen fn each(self) -> Generator[int]:
        for x in self.items:
            yield x    #$ error[E2229]: a borrow of a field of a class object is held across this `yield`

fn main():
    b = Bag()
    for x in b.each():
        println(x)
