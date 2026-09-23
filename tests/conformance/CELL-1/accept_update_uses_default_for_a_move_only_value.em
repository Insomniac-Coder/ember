#$ test: run-pass
#$ rules: CELL-1, OWN-2, TYP-17

from std.core import Default

struct Bag:
    values: Array[i32]

    fn drop(mut self):
        println(self.values.len())

extend Bag implements Default:
    fn default() -> Bag:
        return Bag(Array[i32]())

fn make_bag(count: int) -> Bag:
    values: Array[i32] = Array[i32]()
    i: int = 0
    while i < count:
        values.push(0)
        i = i + 1
    return Bag(values)

fn grow(old: Bag) -> Bag:
    return make_bag(old.values.len() + 1)

fn main():
    cell: Cell[Bag] = Cell(make_bag(2))
    cell.update(grow)
    println(99)
    current = cell.into_inner()
    println(current.values.len())
#$ stdout: 0
#$ 2
#$ 99
#$ 3
#$ 3
