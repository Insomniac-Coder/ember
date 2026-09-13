#$ test: compile-fail
#$ rules: UNS-10, OWN-3
# `UnsafeCell[T]` is never Copy, independently of T.

from std.mem import UnsafeCell

fn main():
    first = UnsafeCell(1)
    second = first
    value = first.into_inner() #$ error[E3040]: moved
    println(value)
    println(second.into_inner())
