#$ test: compile-fail
#$ rules: R5
#$ help: fn reset() changes this:
## R5: `changes this` stands without `reads`: it already lets the method read and change every field
## (E2107).

class Counter:
    count: int = 0
    limit: int = 10

    fn reset() reads limit changes this:    #$ error[E2107]: `reset` says `changes this`, which already lets it read and change every field; write `fn reset() changes this:`
        this.count = this.limit

fn main():
    c = Counter()
    c.reset()
    println(c.count)
