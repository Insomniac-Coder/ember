#$ test: compile-fail
#$ rules: CELL-10, FN-1, BRW-1, DIA-7, DIA-9, DIA-10, DIA-16
#$ help: restructure to a single owner
#$ help: Cell[T]
#$ help: RefCell[T]
# A default-mode value parameter is a shared borrow. Writing through it would
# give the caller and callee potential writers at different times: shape B4,
# not B1's simultaneous mutable-borrow expression. The structural `mut` fix
# must come first; Cell/RefCell/class are later, costed alternatives.

@derive(Copy)
struct Counter:
    pub value: i32

fn increment(counter: Counter):
    counter.value = counter.value + 1  #$ error[E3023]: cannot mutate borrowed parameter `counter`

fn main():
    counter = Counter(1)
    increment(counter)
