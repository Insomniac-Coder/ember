#$ test: run-pass
#$ rules: IV.3, TYP-5
#$ assert-c: contains("for (size_t _i = 0; _i < 4u; ++_i)")
#$ assert-c: contains("ember_panic_bounds")

struct Point:
    x: f32
    y: f32

fn sum(xs: [i32; 4]) -> i32:
    total = 0
    i: usize = 0
    while i < 4:
        total = total + xs[i]
        i = i + 1
    return total

fn swap(p: (i32, f32)) -> (f32, i32):
    return (p.1, p.0)

fn main():
    # A tuple is a value: it is passed, returned and copied whole.
    t = (3, 4.5)
    s = swap(t)
    println(s.0)
    println(s.1)

    # An array literal, indexed and summed through a function.
    xs = [10, 20, 30, 40]
    println(sum(xs))
    println(xs[2])

    # `[value; count]`, and writing through an index.
    ys = [7; 4]
    ys[1] = 100
    println(sum(ys))

    # Arrays and tuples nest, and nest inside structs.
    grid = [[1, 2], [3, 4]]
    println(grid[1][0])

    pair = (Point(1, 2), [5, 6, 7])
    println(pair.0.y)
    println(pair.1[2])

    # An array of tuples.
    ts = [(1, 2), (3, 4)]
    println(ts[1].0)

    # Nested tuples: `n.0.1` is two indices in one float-shaped token.
    n = ((1, 2), (3, (4, 5)))
    println(n.0.1)
    println(n.1.1.0)
#$ stdout: 4.5
#$ 3
#$ 100
#$ 30
#$ 121
#$ 3
#$ 2
#$ 7
#$ 3
#$ 2
#$ 4
