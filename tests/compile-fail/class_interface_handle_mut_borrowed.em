#$ test: compile-fail
#$ rules: OBJ-2, DSP-3, BRW-1
#$ profiles: debug, development, release
#$ error[E2140]: an interface method that changes its object needs `ref mut dyn` access

# A `dyn` parameter is a read-only view (`[OBJ-2]`, the bare interface name `value: Accumulator` is
# `E2173` now); dispatch must not bypass that restriction: a method that changes its object needs
# `ref mut dyn` access.
interface Accumulator:
    fn bump() -> i32

class Counter implements Accumulator:
    value: i32

    fn bump() -> i32:
        this.value = this.value + 1
        return this.value

fn bump_borrowed(value: dyn Accumulator) -> i32:
    return value.bump()

fn main():
    counter = Counter(41)
    println(bump_borrowed(ref counter))
