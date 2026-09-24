#$ test: run-pass
#$ rules: LT-1, FN-1
#$ stdout: [1, 2] 3
#$ hi
#$ 5
# ODR-024 — a borrowed parameter whose type is not `Copy` is the caller's
# place, so a returned view may borrow it: `[LT-1]` rule 2 ties the result to
# it and the caller keeps the argument borrowed while the result lives.

fn head(xs: Array[int]) -> Span[int]:
    return xs[..2]

fn all(xs: Array[int]) -> Span[int]:
    return xs

fn name_of(s: String) -> str:
    return s

fn make() -> Array[int]:
    return [5, 6, 7]

fn main():
    xs = [1, 2, 3]
    println(head(xs), all(xs).len())
    word: String = "hi"
    println(name_of(word))
    println(head(make())[0])
