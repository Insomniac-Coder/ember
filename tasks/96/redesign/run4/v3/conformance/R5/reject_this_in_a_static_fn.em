#$ test: compile-fail
#$ rules: R5
#$ help: fn starting_at(k: int) -> Counter:
## R5: a `static fn` has no `this`; using it there is refused (rules section 3, E1010).

class Counter:
    n: int = 0

    static fn starting_at(k: int) -> Counter:
        return Counter(n=this.n + k)    #$ error[E1010]: `starting_at` is a `static fn`: it is called on the type, `Counter.starting_at(5)`, and has no object; to give it one, remove `static`

fn main():
    c = Counter.starting_at(5)
    println(c.n)
