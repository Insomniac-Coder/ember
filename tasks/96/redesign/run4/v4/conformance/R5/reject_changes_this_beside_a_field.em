#$ test: compile-fail
#$ rules: R5
#$ help: fn reset() changes this:
## R5: `changes this` stands without a field beside it: it already lets the method change every field
## (E2107).

class Counter:
    count: int = 0
    limit: int = 10

    fn reset() changes this, count:    #$ error[E2107]: `reset` says `changes this`, which already lets it change every field; write `fn reset() changes this:` or `fn reset() changes count:`
        this.count = 0
        this.limit = 10

fn main():
    c = Counter()
    c.reset()
    println(c.count)
