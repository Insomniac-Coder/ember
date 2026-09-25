#$ test: run-pass
#$ rules: LEX-15, STD-16
#$ stdout: {1, 2} 3
# ODR-035 — `union` is reserved only where an item would begin `union` and a
# name; everywhere else it is an identifier, so `Set` has its `union` method.

fn union(a: int, b: int) -> int:
    return a + b

fn main():
    s = {1}
    println(s.union({2}), union(1, 2))
