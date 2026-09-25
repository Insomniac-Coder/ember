#$ test: compile-fail
#$ rules: STD-15
# `[STD-15]` (ODR-031) — `drain` takes a range of integers.

fn main():
    xs = [1, 2, 3]
    one = xs.drain(1)                  #$ error[E2020]: `drain` takes a range, not
    names = xs.drain("a".."b")         #$ error[E2020]: `drain` takes a range of integers
