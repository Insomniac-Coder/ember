#$ test: run-fail
#$ rules: CELL-5
# `[CELL-5]` — `borrow_mut` fails while any borrow is active. The shared guard
# from line 8 is live across the `borrow_mut` on line 9, so the counter refuses
# and the panic names the conflicting borrow's location (`borrowed at`).

fn main():
    c: RefCell[i32] = RefCell(1)
    with _a = c.borrow():
        with _b = c.borrow_mut():
            println(1)
#$ panics: RefCell already mutably borrowed (borrowed at
