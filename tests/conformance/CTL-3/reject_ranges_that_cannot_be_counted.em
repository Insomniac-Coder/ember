#$ test: compile-fail
#$ rules: CTL-3
# `[CTL-3]` — a range counts over integers from its start: a `RangeTo` has no
# start, and a range of floats is not counted. `..=b` and `..` have no type in
# the prelude. An expected range type must have the same shape.

fn main():
    upto = ..5
    for i in upto:          #$ error[E2040]: `RangeTo[i64]` cannot be iterated: it has no start
        pass
    unit = 0.5..1.5
    for x in unit:          #$ error[E2020]: cannot count over `f64`
        pass
    a = ..=4                #$ error[E1010]: this range has no type
    b = ..                  #$ error[E1010]: this range has no type
    c: Range[u8] = 0..=3    #$ error[E2020]: expected `Range[u8]`, found `RangeInclusive[i64]`
    d: Range[u8] = 0..300   #$ error[E2010]: the literal `300` does not fit in `u8`
