#$ test: run-pass
#$ rules: TYP-16, FN-1, OBJ-2, DSP-3, EXC-1, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(ember_itable_lookup)
#$ assert-c: !contains(ember_retain)

# Generic interface specialization must survive the `mut I[T]` borrow-before-
# erase boundary, including its concrete one-word dynamic dispatch table.
interface Accumulator[T]:
    fn bump() -> i32

[T]
class Counter implements Accumulator[T]:
    value: i32
    marker: T

    fn bump() -> i32:
        this.value = this.value + 1
        return this.value

fn bump_it(ref mut value: Accumulator[i32]) -> i32:
    return value.bump()

fn main():
    counter = Counter[i32](41, 0)
    println(bump_it(ref mut counter))
