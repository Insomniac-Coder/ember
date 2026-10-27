#$ test: run-pass
#$ rules: R9
#$ stdout: 6
## R9: "Nothing is checked while running: no access words, no borrow flags". Today's `Shared` stops
## the same two changes through two handles while running
## (`tests/conformance/HEAP-5/run_fail_aliased_shared_get_mut.em`); here each line is one change, and
## the emitted C has no access word and no flag test.

struct Counter:
    value: int

fn main():
    counter = shares mut Counter(value=1)
    alias = shares mut counter
    counter.value += 1
    alias.value += 1
    println(counter.value + alias.value)
