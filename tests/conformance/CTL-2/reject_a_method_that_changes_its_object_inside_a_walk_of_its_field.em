#$ test: compile-fail
#$ rules: CTL-2, EXC-15, BRW-18
#$ profiles: debug, development
#$ error[E3020]: `this.grow()` (line 23) changes `this.items`, which this loop walks (line 21); change it after the loop: move line 23, `this.grow()`, below the loop
# `[CTL-2]`, `[EXC-15]` — what a method does to its object is worked out from its body: `grow`
# pushes to `this.items`, so it changes its object, and `walk` calls it inside a loop over
# `this.items`, which would move the buffer the loop reads. The loop holds `this.items` until it
# ends, and the call is refused where it is written. (Until 0.9.15 `grow` could be written `self`,
# a method that only reads, and the push panicked at run time.)

class Bag:
    items: Array[int]

    fn init():
        this.items = [1]

    fn grow():
        this.items.push(owns 9)

    fn walk():
        for x in this.items:
            println(x)
            this.grow()

fn main():
    b = Bag()
    b.walk()
