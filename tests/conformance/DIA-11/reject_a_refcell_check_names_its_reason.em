#$ test: compile-fail
#$ rules: DIA-11, EFF-11, EFF-13
#$ help: change the type
## `[DIA-11]` — a `RefCell`'s borrow check is `requested_by_type` (`[EFF-11]`): the type was chosen
## to check at run time, so the suggestion is to change the type.

@no_alias_checks
fn peek() -> int:
    cell = RefCell(4)
    with g = cell.borrow():  #$ error[E4030]: the check's reason is `requested_by_type`
        return g

fn main():
    println(peek())
