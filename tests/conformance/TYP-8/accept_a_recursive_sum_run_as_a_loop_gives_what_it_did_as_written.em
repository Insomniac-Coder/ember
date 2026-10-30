#$ test: run-pass
#$ rules: TYP-8, RNG-4
#$ profiles: debug, development, release
#$ stdout: 832040 500500 10946 -390 -3
#$ stdout: 5 4 3 2 1 0 1 2 1 0 3 2 1 0 1 5
# `[TYP-8]` — a function whose recursive case returns a call of itself added to a value, and whose
# values the range facts prove never negative (`[RNG-4]`), runs that call as a turn of a loop in an
# optimised build, adding into a running total with every addition checked
# (`ember_analysis::accumulate`): `fib`, `total`, `ways` (unsigned) and `traced`. `shifted` returns
# negative values, so it runs as written. Each gives what the debug profile, which runs them as
# written, gives, and `traced` prints its calls in the order it makes them.

fn fib(n: int) -> int:
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

fn total(n: int) -> int:
    if n < 1:
        return 0
    return n + total(n - 1)

fn ways(n: u32) -> u32:
    if n < 2:
        return 1
    return ways(n - 1) + ways(n - 2)

fn shifted(n: int) -> int:
    if n < 2:
        return n - 5
    return shifted(n - 1) + shifted(n - 2)

fn traced(n: int) -> int:
    print(n, end=" ")
    if n < 2:
        return n
    return traced(n - 1) + traced(n - 2)

fn main():
    println(fib(30), total(1000), ways(20), shifted(10), fib(-3))
    println(traced(5))
