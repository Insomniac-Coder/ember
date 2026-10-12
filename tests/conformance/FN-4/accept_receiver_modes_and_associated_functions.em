#$ test: run-pass
#$ rules: FN-4
#$ stdout: 1
#$ stdout: 1
## `[FN-4]` — a method's receiver is `self`, `mut self` or `owned self`; a
## function in a type body without one is associated, called `Type.name()`.

struct Counter:
    n: int

    fn zero() -> Counter:
        return Counter(0)

    fn get(self) -> int:
        return self.n

    fn bump(mut self):
        self.n += 1

    fn into_count(owned self) -> int:
        return self.n

fn main():
    c = Counter.zero()
    c.bump()
    println(c.get())
    println(c.into_count())
