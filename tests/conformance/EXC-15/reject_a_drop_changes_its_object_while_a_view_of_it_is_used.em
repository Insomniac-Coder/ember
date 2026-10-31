#$ test: compile-fail
#$ rules: EXC-15, DRP-1
#$ profiles: debug, development
#$ error[E3080]: `this.items` is borrowed here and mutably borrowed elsewhere
# `[EXC-15]`, `[DRP-1]` — what a method does to its object is worked out from its body: `reset`
# gives `this.items` a new list, so it changes its object, and `drop` calls it while `v`, a view of
# `this.items`, is still used. The call would free what `v` points into (D-453), and is refused
# where it is written. (Until 0.9.15 `reset` could be written `self`, a method that only reads,
# and the write panicked at run time.)

class Bag:
    items: Array[int]

    fn init():
        this.items = [1, 2, 3, 4, 5, 6, 7, 8]

    fn reset():
        this.items = [0]

    fn drop():
        v = this.items[..]
        this.reset()
        println(v)

fn main():
    b = Bag()
    println(len(b.items))
