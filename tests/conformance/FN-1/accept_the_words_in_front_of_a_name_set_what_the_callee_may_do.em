#$ test: run-pass
#$ rules: FN-1
#$ stdout: 5 2 3
#$ stdout: 9
#$ stdout: [7]
#$ stdout: 4 7
# `[FN-1]` — the word in front of a parameter's name sets what the function may do with it: no word
# reads (a number is copied in), `ref mut` changes the caller's place and `b = v` puts `v` there, a
# `ref mut` view parameter may point the caller's view elsewhere, and `const ref mut` changes the
# caller's value.

struct Box:
    n: int

fn read(a: int, b: Box) -> int:
    return a + b.n

fn put(ref mut b: int, v: int):
    b = v

fn aim(ref mut best: Span[int], other: Span[int]):
    best = other

fn fill(const ref mut b: Box):
    b.n = 9

fn main():
    x = 2
    box = Box(n=3)
    println(read(x, box), x, box.n)
    fill(const ref mut box)
    println(box.n)
    first = [1]
    second = [7]
    s = first.as_span()
    aim(ref mut s, second.as_span())
    println(s)
    n = 4
    put(ref mut n, 7)
    println(4, n)
