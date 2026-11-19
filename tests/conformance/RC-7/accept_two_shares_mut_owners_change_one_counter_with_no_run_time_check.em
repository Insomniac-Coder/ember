#$ test: run-pass
#$ rules: RC-7, RC-1
#$ stdout: 6
## Design test `run_two_shares_mut_owners_change_one_counter_with_no_run_time_check.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
## R9: "Nothing is checked while running: no access words, no borrow flags". Today's `Shared` stops
## the same two changes through two handles while running
## (`tests/conformance/HEAP-5/run_fail_aliased_shared_get_mut.em`); here each line is one change, and
## the emitted C has no access word and no flag test.

struct Counter:
    value: int

fn main():
    shared = shares mut Counter(value=1)
    alias = shares mut shared
    shared.value += 1
    alias.value += 1
    println(shared.value + alias.value)
