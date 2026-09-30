#$ test: compile-fail
#$ rules: STD-19
#$ profiles: debug
# `[STD-19]` (ODR-091) — `rev` is only where an iterator can run backwards: a
# range with no end has no last item, so its iterator has none.

fn main():
    for i in (0..).iter().rev():    #$ error[E1010]: `RangeFromIter[i64]` has no method named `rev`
        println(i)
