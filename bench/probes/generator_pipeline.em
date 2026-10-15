## A generator expression iterated as a value, and a stepped `range` value: the lazy pipeline
## against the C loop that does the same work (bench/probes/generator_pipeline.c).

fn main():
    n = 20000000
    xs: Array[int] = []
    xs.reserve(n)
    seed = 12345
    for _ in range(n):
        seed = (seed * 1103515245 + 12345) % 2147483648
        xs.push(seed % 1000)
    total = 0
    picked = (x * 3 for x in xs if x % 7 != 0)
    for v in picked:
        total += v
    stepped = 0
    counting_down = range(n * 3, 0, -3)
    for v in counting_down:
        stepped += v % 11
    println(total, stepped)
