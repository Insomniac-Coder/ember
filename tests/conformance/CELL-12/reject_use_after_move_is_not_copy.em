#$ test: compile-fail
#$ rules: CELL-12
# `[CELL-12]` — `RefCell[T]` is never `Copy`, whatever `T` is. `d = c` moves;
# borrowing `c` afterwards is a use after move (`[CELL-4]`'s field-derived
# `Copy` explicitly does not reach here).

fn main():
    c: RefCell[i32] = RefCell(1)
    d = c
    with _x = c.borrow():   #$ error[E3050]: borrowed after it has been moved
        println(1)
