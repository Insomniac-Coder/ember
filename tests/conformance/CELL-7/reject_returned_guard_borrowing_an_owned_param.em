#$ test: compile-fail
#$ rules: CELL-7, LT-1
# An `owned` parameter is the callee's own, and its storage ends with the
# frame, so a guard borrowing it cannot be returned (`E3060`).

fn get(owned c: RefCell[i32]) -> Ref[i32]:
    return c.borrow()   #$ error[E3060]: does not live long enough

fn main():
    println(0)
