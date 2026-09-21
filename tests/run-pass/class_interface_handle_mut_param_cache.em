#$ test: run-pass
#$ rules: FN-1, OBJ-2, DSP-3, EXC-1, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c-count: contains("ember_itable_lookup") == 1

# Repeated dispatches through one `mut I` parameter have one stable object
# handle, so [DSP-3] requires one cached TypeInfo interface-table lookup.
interface Accumulator:
    fn bump(mut self) -> i32

class Counter implements Accumulator:
    value: i32

    fn bump(mut self) -> i32:
        self.value = self.value + 1
        return self.value

fn bump_twice(mut value: Accumulator) -> i32:
    value.bump()
    return value.bump()

fn main():
    counter = Counter(40)
    println(bump_twice(counter))
