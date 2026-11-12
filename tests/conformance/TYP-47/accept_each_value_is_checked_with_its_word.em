#$ test: run-pass
#$ rules: TYP-47, FN-1
#$ profiles: debug, development, release
#$ stdout: 2 11 3
#$ kept 2
#$ 4 4
# `[TYP-47]` — each of a pack's values is checked as an ordinary parameter's is, its word
# included: `ref mut values: Ts...` lends each one for the function to change, and `owns values:
# Ts...` takes each one over. Mixed types pass each through its own type's methods.

interface Bump:
    fn bump()
    fn shown() -> int

struct Counter implements Bump:
    n: int

    fn bump():
        this.n += 1

    fn shown() -> int:
        return this.n

struct Tens implements Bump:
    n: int

    fn bump():
        this.n += 10

    fn shown() -> int:
        return this.n

[Ts...: needs Bump]
fn bump_all(ref mut values: Ts...):
    for v in values:
        v.bump()

[Ts...]
fn keep(owns values: Ts...) -> Array[String]:
    kept: Array[String] = []
    for _ in values:
        kept.push(owns f"{values.len()}")
    return kept

[Ts...: needs Bump]
fn total(values: Ts...) -> int:
    sum = 0
    for v in values:
        sum += v.shown()
    return sum

fn main():
    a = Counter(1)
    b = Tens(1)
    c = Counter(2)
    bump_all(ref mut a, ref mut b, ref mut c)
    println(a.n, b.n, c.n)
    name: String = "one"
    other: String = "two"
    kept = keep(owns name, owns other)
    println("kept", kept.len())
    println(total(Counter(1), Counter(3)), total(Tens(4)))
