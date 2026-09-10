#$ test: run-pass
#$ rules: CELL-7
# `[CELL-7]` — a returned `Ref` keeps the caller's cell borrowed via elision:
# the `ref` parameter is view-typed, so the guard's region ties to it
# (`[LT-1]`). The caller reads through the guard (`println(g)` prints the
# contents, 41) and drops it at the block end. No `L3011` here: the guard's
# loan lives in the callee, and the caller's `println` cannot reach the cell.

fn get(mut c: RefCell[i32]) -> Ref[i32]:
    return c.borrow()

fn main():
    c: RefCell[i32] = RefCell(41)
    with g = get(c):
        println(1)
        println(g)
#$ stdout: 1
#$ 41
