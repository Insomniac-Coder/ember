#$ test: run-pass
#$ rules: R9
#$ stdout: 6
## R9: "Nothing is checked while running: no access words, no borrow flags". Today's `Shared` stops
## the same two changes through two handles while running
## (`tests/conformance/HEAP-5/run_fail_aliased_shared_get_mut.em`); here each line is one change, and
## the emitted C has no access word and no flag test. `main` changes the counter through its own links,
## so it says `changes Counter`, as verdict 64's `fn main() changes Clock:` does.

struct Counter:
    value: int

fn main() changes Counter:
    shared = shares mut Counter(value=1)
    alias = shares mut shared
    shared.value += 1
    alias.value += 1
    println(shared.value + alias.value)
