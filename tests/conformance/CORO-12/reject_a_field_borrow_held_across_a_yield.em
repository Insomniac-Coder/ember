#$ test: compile-fail
#$ rules: CORO-12
#$ profiles: debug
# `[CORO-12]` — a class's generator method holds its object as a handle, each access to it in the
# body checked on its own, so a long-term access to a field of the object live across a `yield` is
# `E2229`: other code may run while the generator is suspended. (0.9.10 also refused a generator
# method written `mut self`, which 0.9.15 cannot write: what a method does to its object is worked
# out from its body.)

class Bag:
    items: Array[int] = []

    gen fn each() -> Generator[int]:
        for x in this.items:
            yield x    #$ error[E2229]: a borrow of a field of a class object is held across this `yield`

fn main():
    b = Bag()
    for x in b.each():
        println(x)
