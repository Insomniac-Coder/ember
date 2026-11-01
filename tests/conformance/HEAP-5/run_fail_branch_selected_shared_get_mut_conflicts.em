#$ test: run-fail
#$ rules: HEAP-3, HEAP-5, HEAP-6, EXC-1, EXC-2, RC-5
#$ profiles: debug, development, release
#$ panics: exclusivity violation: overlapping access

struct Counter:
    value: i32

fn open(ref mut owner: Shared[Counter]) -> ref mut Counter from owner:
    return owner.get_mut()

fn main():
    first = Shared(Counter(1))
    first_alias = first
    second = Shared(Counter(2))
    choose_first = true
    chosen = open(ref mut first) if choose_first else open(ref mut second)
    # The selected owner is `first`; opening the unrelated `second` here would
    # be a false positive, while this alias must still conflict at runtime.
    conflicting = first_alias.get_mut()
    println(chosen.value + conflicting.value)
