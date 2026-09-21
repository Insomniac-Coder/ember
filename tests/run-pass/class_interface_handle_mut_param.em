#$ test: run-pass
#$ rules: FN-1, OBJ-2, DSP-3, EXC-1, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(ember_itable_lookup)
#$ assert-c: contains(ember_access_begin_write)
#$ assert-c: contains(ember_access_end_write)
#$ assert-c: !contains(ember_retain)

# A `mut I` parameter is an inout borrow of the same one-word class-interface
# handle. The method call must keep the concrete class access interval.
interface Accumulator:
    fn bump(mut self) -> i32

class Counter implements Accumulator:
    value: i32

    fn bump(mut self) -> i32:
        self.value = self.value + 1
        return self.value

fn bump_it(mut value: Accumulator) -> i32:
    return value.bump()

fn main():
    counter = Counter(41)
    println(bump_it(counter))
