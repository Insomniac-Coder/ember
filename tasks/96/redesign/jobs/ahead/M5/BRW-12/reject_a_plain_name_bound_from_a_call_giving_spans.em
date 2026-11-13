# Clause BRW-12.7: "before a `MutSpan`, or a tuple of them, a call gives it [`ref mut`] says the names
# bound from it may write the spans' elements" -- without it they only read (`[SPN-3]`: "without
# `ref mut` the names only read (`E3023`, whose help gives that line)").
#$ test: compile-fail
#$ rules: BRW-12, SPN-3, BRW-15
#$ help: left, right = ref mut s.split_at(2)
# The help is the line `[SPN-3]` says it gives; the message text is not given: code only.

fn main():
    xs = [1, 2, 3, 4]
    s = ref mut xs.as_mut_span()
    left, right = s.split_at(2)
    left[0] = 7    #$ error[E3023]
    println(right[0])
