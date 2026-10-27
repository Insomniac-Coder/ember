#$ test: compile-fail
#$ rules: R5
#$ help: return Counter(n=k)
## R5: a `static fn` has no `this`; using it there is refused (rules section 3, E1010). The way out
## keeps `main`'s `Counter.starting_at(5)` compiling (R18), which removing `static` would not.

class Counter:
    n: int = 0

    static fn starting_at(k: int) -> Counter:
        return Counter(n=this.n + k)    #$ error[E1010]: `starting_at` is a `static fn`: it is called on the type, `Counter.starting_at(5)`, and has no object, so no `this`; build the counter from `k`: `return Counter(n=k)`

fn main():
    c = Counter.starting_at(5)
    println(c.n)
