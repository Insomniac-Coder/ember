#$ test: run-fail
#$ rules: HEAP-3, HEAP-5, HEAP-6, EXC-1, EXC-2, RC-5
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: overlapping access

struct Counter:
    value: i32

@borrows(first, second)
fn choose_mut(mut first: Shared[Counter], mut second: Shared[Counter], choose_first: bool) -> ref mut Counter:
    if choose_first:
        return first.get_mut()
    return second.get_mut()

fn main():
    first = Shared(Counter(1))
    first_alias = first
    second = Shared(Counter(2))
    chosen = choose_mut(first, second, true)
    # The selected owner is `first`; opening the unrelated `second` here would
    # be a false positive, while this alias must still conflict at runtime.
    conflicting = first_alias.get_mut()
    println(chosen.value + conflicting.value)
