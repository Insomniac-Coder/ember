#$ test: run-pass
#$ rules: HEAP-3, HEAP-5, HEAP-6, RC-5, EXC-1, EXC-2
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(ember_access_begin_write)
#$ assert-c: contains(ember_access_end_write)

struct Counter:
    value: i32

fn main():
    shared = Shared(Counter(40))
    alias = shared

    first = shared.get_mut()
    first.value = first.value + 1

    # The first mutable loan ends at its last use, so another aliased owner
    # may start a new dynamic access without a spurious conflict.
    second = alias.get_mut()
    second.value = second.value + 1
    println(second.value)
