#$ test: compile-fail
#$ rules: CELL-10, FN-1, BRW-1, SPN-1, SPN-3, DIA-7, DIA-9
#$ help: restructure to a single owner
#$ help: RefCell[T]
# Mutable views are created by the centralized `view_of` path. That path must
# enforce the source parameter's shared mode; otherwise `as_mut_span()` would
# expose writable storage from a default-mode Array parameter.

fn expose(values: Array[i32]):
    view: MutSpan[i32] = values.as_mut_span()  #$ error[E3023]: cannot mutate borrowed parameter `values`
    view[0] = 2

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    expose(values)
