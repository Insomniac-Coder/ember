#$ test: run-pass
#$ rules: CELL-1, CELL-2, TYP-14
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains("(*((em_Cell_i32*)_1)).value = ")
#$ assert-c: !contains("ember_cell")

# Cell's explicit interior-mutation API remains available through `ref Cell`.
# Auto-dereference reaches the Cell receiver, while Cell—not a mutable source
# reference—owns the permitted mutation boundary.
fn bump(cell: ref Cell[i32]):
    cell.set(cell.get() + 1)

fn main():
    cell = Cell(41)
    bump(ref cell)
    println(cell.get())
