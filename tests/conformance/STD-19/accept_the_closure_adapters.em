#$ test: run-pass
#$ rules: STD-19, CLO-14, STD-9
#$ stdout: 1 2 3 4 5 6 [2, 4, 6, 8, 10, 12] [2, 4, 6] 3 3 [1, 2, 3] 81 6
#$ stdout: [2, 3, 4, 5, 6, 7]
# `[STD-19]` — `map`, `filter`, `filter_map`, `take_while`, `skip_while` and
# `inspect`, each an adapter holding the callable it was given (a `[CLO-14]`
# bound, so a lambda that borrows keeps its borrow), run by consumers and
# through an `Iterable`'s form (`xs.map(f)` is `xs.iter().map(f)`). None
# existed.

fn main():
    xs = [1, 2, 3, 4, 5, 6]
    k = 10
    doubled = xs.iter().map(owns fn(x: ref int) => x * 2).to_array()
    evens = xs.iter().filter(owns fn(x: ref int) => x % 2 == 0).copy_each().to_array()
    small = xs.iter().take_while(owns fn(x: ref int) => x < 4).count()
    rest = xs.iter().skip_while(owns fn(x: ref int) => x < 4).count()
    halves = xs.iter().filter_map(owns fn(x: ref int) => Some(x // 2) if x % 2 == 0 else None).to_array()
    shifted = xs.iter().map(owns fn(x: ref int) => x + k).fold(0, owns fn(a: int, b: int) => a + b)
    total = xs.iter().inspect(owns fn(x: ref int) => print(x, end=" ")).count()
    println(doubled, evens, small, rest, halves, shifted, total)
    println(xs.map(fn(x: ref int) => x + 1).to_array())
