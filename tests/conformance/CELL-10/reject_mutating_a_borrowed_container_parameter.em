#$ test: compile-fail
#$ rules: CELL-10, FN-1, BRW-1, DIA-7, DIA-9
#$ help: restructure to a single owner
#$ not-help: RefCell
# A method with `mut self` is another write through the default-mode parameter.
# The type checker must retain parameter-mode provenance through method-call
# lowering; checking assignment alone would leave this path unsound. Calls may
# contain simultaneous sibling accesses, so this site offers only the primary
# structural repair until the complete call proves otherwise.

fn append_one(values: Array[i32]):
    values.push(1)  #$ error[E3023]: cannot mutate borrowed parameter `values`

fn main():
    values: Array[i32] = Array[i32]()
    append_one(values)
