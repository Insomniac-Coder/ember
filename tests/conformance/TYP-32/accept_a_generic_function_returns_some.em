#$ test: run-pass
#$ rules: TYP-32, TYP-16
#$ stdout: [3, 6, 9]
#$ stdout: [1.5, 3.0]
# `[TYP-32]` — a generic function returning `some I`: each instance returns its own concrete type
# (`scaled[int]`'s and `scaled[float]`'s maps differ in their item and their lambda), and the
# caller of each sees the `I` its arguments make, `Iterator[Item = int]` or `[Item = float]`.

fn scaled[T: Copy + Mul[Output = T]](xs: Span[T], k: T) -> some Iterator[Item = T]:
    return xs.iter().copied().map(owned fn(x) => x * k)

fn main():
    a = [1, 2, 3]
    println(scaled(a.as_span(), 3).to_array())
    f = [0.5, 1.0]
    println(scaled(f.as_span(), 3.0).to_array())
