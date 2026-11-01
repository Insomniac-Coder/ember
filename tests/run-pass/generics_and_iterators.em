#$ test: run-pass
#$ rules: TYP-16, TYP-17, TYP-18, IFC-4, CTL-1, CTL-4

interface Shape:
    fn area() -> f32

# `[IFC-4]` — an associated type: each implementer says what `Item` is.
interface Iterator:
    type Item
    fn next() -> Option[Item]

struct Square implements Shape:
    side: f32
    fn area() -> f32:
        return this.side * this.side

struct Circle implements Shape:
    r: f32
    fn area() -> f32:
        return this.r * this.r * 3.0

struct Counting:
    at: i32
    stop: i32

extend Counting implements Iterator:
    type Item = i32
    fn next() -> Option[i32]:
        if this.at >= this.stop:
            return None
        v = this.at
        this.at = this.at + 1
        return Some(v)

# `[TYP-16]`, `[TYP-18]` — one instantiation per type, inferred from the
# arguments.
[T]
fn identity(x: T) -> T:
    return x

[T]
fn pick(a: T, b: T, first: bool) -> T:
    if first:
        return a
    return b

# `[TYP-17]` — only what the bound provides may be used, and `area` is
# provided by `Shape`.
[T: needs Shape]
fn total_area(a: T, b: T) -> f32:
    return a.area() + b.area()

fn main():
    # The same function, instantiated at three different types.
    println(identity(41) + 1)
    println(identity(1.5))
    println(pick(10, 20, true))
    println(pick(1.5, 2.5, false))

    println(total_area(Square(2.0), Square(3.0)))
    println(total_area(Circle(1.0), Circle(2.0)))

    # `[CTL-1]` — `for` over anything that provides `next()`.
    total: i32 = 0
    for x in Counting(0, 5):
        total = total + x
    println(total)

    # `[CTL-4]` — `else` runs when the iterator is exhausted, not on `break`.
    for _x in Counting(0, 3):
        total = total + 0
    else:
        println(111)

    for _x in Counting(0, 3):
        break
    else:
        println(-1)

    # And over a collection, which is a counted loop over its indices.
    xs: Array[int] = Array()
    for i in 0..5:
        xs.push(owns (i * i))
    sum = 0
    for x in xs:
        sum = sum + x
    println(sum)
#$ stdout: 42
#$ 1.5
#$ 10
#$ 2.5
#$ 13.0
#$ 15.0
#$ 10
#$ 111
#$ 30
