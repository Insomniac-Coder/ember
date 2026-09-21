#$ test: compile-fail
#$ rules: OBJ-2, DSP-3, BRW-1
#$ profiles: debug, release, shipping
#$ error[E3023]: cannot mutate borrowed parameter `value`

# An interface-handle parameter without `mut` is borrowed just like an
# ordinary class-handle parameter; dispatch must not bypass that restriction.
interface Accumulator:
    fn bump(mut self) -> i32

class Counter implements Accumulator:
    value: i32

    fn bump(mut self) -> i32:
        self.value = self.value + 1
        return self.value

fn bump_borrowed(value: Accumulator) -> i32:
    return value.bump()

fn main():
    counter = Counter(41)
    println(bump_borrowed(counter))
