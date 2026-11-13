#$ test: compile-fail
#$ rules: UNS-2
# `[UNS-2]` — `unsafe` does not turn off the checks of Part VII: inside the block a name handed over
# is still not used again (`E3040`), and a change while a view of the value is used is still refused
# (`E3021`).

fn main():
    xs: Array[int] = [1, 2, 3]
    p = xs.as_mut_span().as_mut_ptr()
    s = String("a")
    # SAFETY: `xs` holds three elements, and the write is to the first.
    unsafe:    #$ warning[L3010]: larger than necessary
        p.write(9)
        t = s
        println(s)    #$ error[E3040]: `s` was handed over to `t` on line 14 and is used on line 15
        v = xs[..]
        xs.push(owns 4)    #$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
        println(v)
        println(t)
