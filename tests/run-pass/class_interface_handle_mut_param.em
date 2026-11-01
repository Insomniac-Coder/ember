#$ test: run-pass
#$ rules: FN-1, OBJ-2, DSP-3, EXC-1, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(ember_itable_lookup)
#$ assert-c: contains(ember_object_begin_write)
#$ assert-c: contains(ember_object_end_write)
#$ assert-c: !contains(ember_retain)

# A `mut I` parameter is an inout borrow of the same one-word class-interface
# handle. The method call must keep the concrete class access interval.
interface Accumulator:
    fn bump() -> i32

class Counter implements Accumulator:
    value: i32

    fn bump() -> i32:
        this.value = this.value + 1
        return this.value

fn bump_it(ref mut value: Accumulator) -> i32:
    return value.bump()

fn main():
    counter = Counter(41)
    println(bump_it(ref mut counter))
