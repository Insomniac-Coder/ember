#$ test: compile-fail
#$ rules: DRP-6, OWN-3
# Ownership belongs to the Box, not to T's Copy status.

fn main():
    first: Box[i32] = Box(1)
    second = first
    println(second.get())
    println(first.get()) #$ error[E3050]: `first` is borrowed after it has been moved out of
