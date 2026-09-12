#$ test: compile-fail
#$ rules: CELL-10, FN-1, BRW-1, DIA-7, DIA-9
#$ help: restructure to a single owner
#$ help: RefCell[T]
# Explicit `ref mut` is a write-capable access, just like assignment. This
# probe ensures the shared parameter mode is enforced before MIR borrow
# collection and is not limited to assignment syntax.

struct Counter:
    pub value: i32

fn expose(counter: Counter):
    value: ref mut i32 = ref mut counter.value  #$ error[E3023]: cannot mutate borrowed parameter `counter`
    value = 2

fn main():
    counter = Counter(1)
    expose(counter)
