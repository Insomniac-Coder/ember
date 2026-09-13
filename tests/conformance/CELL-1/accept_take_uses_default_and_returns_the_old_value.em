#$ test: run-pass
#$ rules: CELL-1, TYP-17

from std.core import Default

struct Bag:
    values: Array[i32]

extend Bag implements Default:
    fn default() -> Bag:
        return Bag(Array[i32]())

fn make_bag() -> Bag:
    values: Array[i32] = Array[i32]()
    values.push(10)
    values.push(20)
    return Bag(values)

fn main():
    cell: Cell[Bag] = Cell(make_bag())
    old = cell.take()
    println(old.values.len())
    replacement = cell.into_inner()
    println(replacement.values.len())
#$ stdout: 2
#$ 0
