#$ test: run-pass
#$ rules: STD-19, TYP-16
#$ stdout: [3, 6, 9]

## D-736 — an interface's default method (`map[R, F]`) copied onto an instance that holds the
## caller's parameters (`SpanIter[T]` in a generic body) kept its own `R` in `T`'s slot, so `Item`
## read as `R`: "`map` cannot take `fn(ref R) -> T` where it expects `fn(ref R) -> R`". Its own
## parameters now come after the caller's, as a generic type's methods' do (D-280).

fn scaled[T: Copy + Mul[Output = T]](xs: Span[T], k: T) -> Array[T]:
    return xs.iter().map(fn(x) => x * k).to_array()

fn main():
    a = [1, 2, 3]
    println(scaled(a.as_span(), 3))
