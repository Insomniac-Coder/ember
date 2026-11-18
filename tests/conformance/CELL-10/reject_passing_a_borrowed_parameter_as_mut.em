#$ test: compile-fail
#$ rules: CELL-10, FN-1, FN-2a, BRW-1, DIA-7, DIA-9
#$ help: take it as `ref mut counter: Counter` on `forward`'s first line
#$ not-help: RefCell
# A mutable argument is formed after ordinary expression checking. Preserving
# the default parameter's shared-borrow mode to this boundary prevents the ABI
# representation from turning a borrowed value into an apparent local owner.

struct Counter:
    pub value: i32

fn increment(ref mut counter: Counter):
    counter.value = counter.value + 1

fn forward(counter: Counter):
    increment(ref mut counter)  #$ error[E3027]: `counter` only reads the caller's counter, so it cannot be given to `increment` as `ref mut`

fn main():
    counter = Counter(1)
    forward(counter)
