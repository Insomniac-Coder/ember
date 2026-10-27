#$ test: run-pass
#$ rules: FN-4
#$ stdout: 5
## design rules: R5
## R5, verdict 7: a function in a type body written `static fn` has no `this` and is called on the
## type.

class Counter:
    n: int = 0

    static fn starting_at(k: int) -> Counter:
        return Counter(n=k)

fn main():
    c = Counter.starting_at(5)
    println(c.n)
