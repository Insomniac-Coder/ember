#$ test: compile-fail
#$ rules: R5
#$ help: fn reset() changes this:
## R5, verdict 6: what a method does to its object is said after its parameters (`changes this`);
## `mut this` is `mut` alone on a name, which says "may change" but not whose (rules section 3, E0113).

class Counter:
    count: int = 0

    fn reset(mut this):    #$ error[E0113]: `mut this`: write `changes this` after the brackets: `fn reset() changes this:`
        this.count = 0

fn main():
    c = Counter()
    c.reset()
    println(c.count)
