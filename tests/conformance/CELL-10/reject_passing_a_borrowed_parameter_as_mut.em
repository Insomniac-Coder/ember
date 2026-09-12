#$ test: compile-fail
#$ rules: CELL-10, FN-1, BRW-1, DIA-7, DIA-9
#$ help: restructure to a single owner
#$ not-help: RefCell
# A mutable argument is formed after ordinary expression checking. Preserving
# the default parameter's shared-borrow mode to this boundary prevents the ABI
# representation from turning a borrowed value into an apparent local owner.

struct Counter:
    pub value: i32

fn increment(mut counter: Counter):
    counter.value = counter.value + 1

fn forward(counter: Counter):
    increment(counter)  #$ error[E3023]: cannot mutate borrowed parameter `counter`

fn main():
    counter = Counter(1)
    forward(counter)
