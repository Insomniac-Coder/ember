#$ test: run-pass
#$ rules: CELL-1, CELL-2, TYP-14
#$ profiles: debug, development, release
#$ stdout: 43
#$ assert-c: contains("(*((em_3std4core4CellG2c2a47066b4fc729*)cell)).value = ")
#$ assert-c: !contains("ember_cell")

# Cell's explicit interior-mutation API remains available through `ref Cell`.
# Auto-dereference reaches the Cell receiver, while Cell—not a mutable source
# reference—owns the permitted mutation boundary.
fn bump(cell: ref Cell[i32]):
    cell.set(cell.get() + 1)

fn main():
    cell: Cell[i32] = Cell(41)
    bump(ref cell)
    bump(ref cell)
    println(cell.get())
