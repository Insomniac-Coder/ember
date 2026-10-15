#$ test: compile-fail
#$ rules: DRP-4
#$ error[E4001]: `@noalloc` function `Bag.drop` reaches an allocation
# `[DRP-4]` — `@noalloc` on a type's `drop` is honoured: the destructor's own body is checked.

class Bag:
    n: int

    @noalloc
    fn drop(mut self):
        trail = [self.n]
        println(trail.len())

fn main():
    b = Bag(1)
    println(b.n)
