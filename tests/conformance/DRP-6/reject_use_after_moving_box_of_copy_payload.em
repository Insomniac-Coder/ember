#$ test: compile-fail
#$ rules: DRP-6, OWN-3
# Ownership belongs to the Box, not to T's Copy status.

fn main():
    first: Box[i32] = Box(1)
    second = first
    println(second.get())
    println(first.get()) #$ error[E3040]: `first` was handed over to `second` on line 7 and is used on line 9; to keep both, write `second = copy of first` (a copy of your own) or `second = shares first` (two owners of one value)
