#$ test: run-pass
#$ rules: CELL-1, CELL-2, TYP-14
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ 42
#$ assert-c: contains("(*((em_Cell_i32*)_1)).value = ")
#$ assert-c: !contains("ember_cell")

# `replace` is a shared Cell operation: it transfers the previous payload to
# its caller and stores the new payload through the Cell boundary.
fn replace(cell: ref Cell[i32]) -> i32:
    return cell.replace(42)

fn main():
    cell: Cell[i32] = Cell(7)
    println(replace(ref cell))
    println(cell.get())
