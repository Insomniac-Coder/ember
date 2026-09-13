#$ test: run-pass
#$ rules: EXP-1, CELL-1
# A compiler-known mutating method still follows ordinary call evaluation:
# evaluate the indexed receiver place before the value argument, then perform
# Cell's distinct store-before-drop replacement.

fn choose() -> usize:
    println(1)
    return 0

fn make_value() -> i32:
    println(2)
    return 7

fn main():
    cells: Array[Cell[i32]] = Array[Cell[i32]]()
    cells.push(Cell(0))
    cells[choose()].set(make_value())
    println(cells[0].get())
#$ stdout: 1
#$ 2
#$ 7
