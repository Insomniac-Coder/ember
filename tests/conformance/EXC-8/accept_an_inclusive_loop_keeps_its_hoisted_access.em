#$ test: run-pass
#$ rules: EXC-8, CTL-3
#$ profiles: debug, release, shipping
#$ stdout: 4 3

# `[EXC-8]` (D-525) — an inclusive loop whose end is not known to be below
# its type's top tests for its last turn before it steps; that test leaves
# the loop too, so the access hoisted out of the loop ends on that way out as
# on the head's, and the call after the loop finds the counter free.

class Counter:
    value: i32

    fn bump(mut self):
        self.value = self.value + 1

fn main():
    counter = Counter(0)
    alias = counter
    seed: Array[int] = [7]
    n = seed.len() + 1
    for i in 0..=n:
        counter.bump()
    counter.bump()
    other = Counter(0)
    also = other
    for i in (int.MAX - 2) ..= int.MAX:
        other.bump()
    println(alias.value, also.value)
