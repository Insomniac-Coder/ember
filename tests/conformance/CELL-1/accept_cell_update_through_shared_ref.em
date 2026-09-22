#$ test: run-pass
#$ rules: CELL-1, CELL-2, TYP-14
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains("(*((em_Cell_i32*)_1)).value = ")
#$ assert-c: !contains("ember_cell")

# `update` reads the old payload, applies the callback, then stores the result
# through Cell's explicit shared interior-mutation boundary.
fn add_two(value: i32) -> i32:
    return value + 2

fn update(cell: ref Cell[i32]):
    cell.update(add_two)

fn main():
    cell = Cell(40)
    update(ref cell)
    println(cell.get())
