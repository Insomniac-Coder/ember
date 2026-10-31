#$ test: run-pass
#$ rules: FN-4
#$ stdout: 1
#$ stdout: 1
## `[FN-4]` — a method's receiver is `self`, `mut self` or `owned self`; a
## function in a type body without one is associated, called `Type.name()`.

struct Counter:
    n: int

    static fn zero() -> Counter:
        return Counter(0)

    fn get() -> int:
        return this.n

    fn bump():
        this.n += 1

    fn into_count() owns this -> int:
        return this.n

fn main():
    c = Counter.zero()
    c.bump()
    println(c.get())
    println(c.into_count())
