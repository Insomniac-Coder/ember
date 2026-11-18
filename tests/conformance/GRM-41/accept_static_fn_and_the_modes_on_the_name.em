#$ test: run-pass
#$ rules: GRM-41, FN-1, FN-4
#$ stdout: 5
#$ stdout: 6
#$ stdout: 3
## `[GRM-41]` — `static fn` in a type body has no object and is called on the type; a parameter's word is
## `ref`, `ref mut`, `const ref mut` or `owns`, written on its name.

struct Counter:
    n: int = 0

    static fn starting_at(k: int) -> Counter:
        return Counter(n=k)

    fn bump():
        this.n += 1

fn grow(ref mut c: Counter):
    c.bump()

fn size(ref xs: Array[int]) -> int:
    return xs.len()

fn main():
    c = Counter.starting_at(5)
    println(c.n)
    grow(ref mut c)
    println(c.n)
    println(size([1, 2, 3]))
