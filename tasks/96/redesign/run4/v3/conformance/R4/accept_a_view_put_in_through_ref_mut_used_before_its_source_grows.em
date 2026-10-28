#$ test: run-pass
#$ rules: R4, R6, R8
#$ stdout: 3
#$ 4
## R4, R6: the nearest allowed program: `best` is used before `b` grows, and `b` is read again after.

fn pick_longer(ref mut best: Span[int], other: Span[int]):
    if other.len() > best.len():
        best = other

fn main():
    a = [1]
    b = [1, 2, 3]
    best = a.as_span()
    pick_longer(ref mut best, b.as_span())
    println(best.len())
    b.push(4)
    println(b.len())
