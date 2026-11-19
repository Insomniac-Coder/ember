#$ test: compile-fail
#$ rules: CELL-7, LT-1
# An `owned` parameter is the callee's own, and its storage ends with the
# frame, so a guard borrowing it cannot be returned (`E3060`).

fn get(owns c: RefCell[i32]) -> Ref[i32]:
    return c.borrow()   #$ error[E3063]: which ends when the function returns

fn main():
    println(0)
