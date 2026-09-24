#$ test: compile-pass
#$ rules: LT-1b, DIA-14, MAN-3
# `L3014` is about the declaration: an interface's default method gets it
# once, not again for each type that takes the default.

interface Picker:
    fn pick(a: Array[int], b: Array[int]) -> Span[int]:    #$ warning[L3014]: return region is the intersection of 2 parameters
        return a

struct A implements Picker:
    x: int

struct B implements Picker:
    x: int

fn main():
    pass
