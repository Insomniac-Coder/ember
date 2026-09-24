#$ test: run-pass
#$ rules: CELL-7, LT-1
#$ stdout: 7
# ODR-024 — a borrowed `RefCell` parameter is the caller's cell, passed by
# address, so a guard borrowing it may be returned: `[LT-1]` rule 2 ties the
# result to that parameter and the caller keeps its cell borrowed.

fn get(c: RefCell[i32]) -> Ref[i32]:
    return c.borrow()

fn main():
    cell: RefCell[i32] = RefCell(7)
    with g = get(cell):
        println(g + 0)
