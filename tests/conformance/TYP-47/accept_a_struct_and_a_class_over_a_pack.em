#$ test: run-pass
#$ rules: TYP-47, TYP-27, MONO-1
#$ profiles: debug, development, release
#$ stdout: 1 2.5 x
#$ 9 more
#$ 3
#$ 5
#$ 7 4.5 y
#$ 8
#$ 6
# `[TYP-47]` — a `struct` or `class` takes a pack as a function does, its last type parameter
# (`[Cs...]` above `struct Row:`), and a type written over the pack is formed once it is spelled
# out (`[TYP-27]`): `(int, Cs...)` with two types is a tuple of three, with one a pair and with none
# the `int` itself; `fn(Cs...)` takes one parameter per type. The pack's types are written
# (`Row[f64, StrView]`, `Bag[Cs...]`) or met by the fields' values. Each instance is made for its
# exact types (`[MONO-1]`).

[Cs...]
struct Row:
    id: int
    cells: (int, Cs...)

[Cs...]
class Bag:
    count: int

    fn more() -> int:
        return this.count + 1

[Cs...]
fn call(f: fn(Cs...) -> int, values: Cs...) -> int:
    return f(values...)

fn add(a: int, b: int) -> int:
    return a + b

[Cs...]
fn tagged(id: int) -> Bag[Cs...]:
    return Bag[Cs...](count = id)

fn main():
    wide = Row[f64, StrView](id = 1, cells = (1, 2.5, "x"))
    println(wide.cells.0, wide.cells.1, wide.cells.2)
    pair = Row[StrView](id = 3, cells = (9, "more"))
    println(pair.cells.0, pair.cells.1)
    bag = Bag[int, StrView](count = 2)
    println(bag.more())
    println(call(add, 2, 3))
    seven: int = 7
    half: f64 = 4.5
    inferred = Row(id = 4, cells = (seven, half, "y"))
    println(inferred.cells.0, inferred.cells.1, inferred.cells.2)
    eight: int = 8
    bare = Row(id = 5, cells = eight)
    println(bare.cells)
    println(tagged[int, StrView](5).more())
