#$ test: compile-fail
#$ rules: R5
#$ help: write `fn reset():`
## R5: a method changes its object through `this` with no word (verdict 86); `mut this` is `mut` alone on
## a name, which says "may change" but not whose (rules section 3, E0113, its message word for word).

class Counter:
    count: int = 0

    fn reset(mut this):    #$ error[E0113]: `mut this`: a method changes its object through `this` with no word
        this.count = 0

fn main():
    c = Counter()
    c.reset()
    println(c.count)
