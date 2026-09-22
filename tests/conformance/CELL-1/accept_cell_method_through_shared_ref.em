#$ test: run-pass
#$ rules: CELL-1, CELL-2, TYP-14
#$ profiles: debug, release, shipping
#$ stdout: 7

# A compiler-known Cell operation must still be found through an ordinary
# shared reference; auto-dereference reaches the Cell, not its payload.
fn read_cell(cell: ref Cell[i32]) -> i32:
    return cell.get()

fn main():
    cell = Cell(7)
    println(read_cell(ref cell))
