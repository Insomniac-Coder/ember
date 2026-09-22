#$ test: run-pass
#$ rules: TYP-16, FN-1, OBJ-2, DSP-3, EXC-1, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(ember_itable_lookup)
#$ assert-c: !contains(ember_retain)

# Generic interface specialization must survive the `mut I[T]` borrow-before-
# erase boundary, including its concrete one-word dynamic dispatch table.
interface Accumulator[T]:
    fn bump(mut self) -> i32

class Counter[T] implements Accumulator[T]:
    value: i32
    marker: T

    fn bump(mut self) -> i32:
        self.value = self.value + 1
        return self.value

fn bump_it(mut value: Accumulator[i32]) -> i32:
    return value.bump()

fn main():
    counter = Counter[i32](41, 0)
    println(bump_it(counter))
