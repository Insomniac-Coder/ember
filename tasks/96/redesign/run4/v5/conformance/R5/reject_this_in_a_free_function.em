#$ test: compile-fail
#$ rules: MOD-3
#$ help: fn bump(c: Counter) -> int:
## design rules: R5
## R5: only a method has an object; `this` in a free function is refused (rules section 3's "a free
## function", E1010; completeness G7).

struct Counter:
    n: int = 0

fn bump() -> int:
    return this.n + 1    #$ error[E1010]: `bump` is not a method, so it has no object and no `this`; take the counter as a parameter: write `fn bump(c: Counter) -> int:`, `return c.n + 1` in place of this line, and call it `bump(c)`

fn main():
    c = Counter(n=1)
    println(bump())
