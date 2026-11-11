#$ test: compile-fail
#$ rules: CELL-12
# `[CELL-12]` — `RefCell[T]` is never `Copy`, whatever `T` is. `d = c` moves;
# borrowing `c` afterwards is a use after move (`[CELL-4]`'s field-derived
# `Copy` explicitly does not reach here).

fn main():
    c: RefCell[i32] = RefCell(1)
    _d = c
    with _x = c.borrow():   #$ error[E3040]: `c` was handed over to `_d` on line 9 and is used on line 10; to keep both, write `_d = copy of c` (a copy of your own) or `_d = shares c` (two owners of one value)
        println(1)
