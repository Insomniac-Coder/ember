#$ test: run-pass
#$ rules: FN-1, OBJ-2, DSP-3, EXC-1, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c-count: contains("ember_itable_lookup") == 1

# Repeated dispatches through one `mut I` parameter have one stable object
# handle, so [DSP-3] requires one cached TypeInfo interface-table lookup.
interface Accumulator:
    fn bump() -> i32

class Counter implements Accumulator:
    value: i32

    fn bump() -> i32:
        this.value = this.value + 1
        return this.value

fn bump_twice(ref mut value: Accumulator) -> i32:
    value.bump()
    return value.bump()

fn main():
    counter = Counter(40)
    println(bump_twice(counter))
