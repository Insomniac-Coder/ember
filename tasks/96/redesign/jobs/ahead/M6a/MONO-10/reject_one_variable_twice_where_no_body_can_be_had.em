# Clause MONO-10.4: "Where no body can be had, a call through a function value, an interface, `dyn`
# or `virtual` method, or of an `extern` function, a call that names one whole variable twice is
# `E3022`"
#$ test: compile-fail
#$ rules: MONO-10, BRW-11
# The function value's message is the spec's, with this file's line numbers; the `virtual` method's
# has no text in the spec: code only. The `extern` case needs `unsafe import c` and a C file (M12):
# not here.

fn add_both(ref mut a: int, ref mut b: int):
    a += 1
    b += 1

open class Adder:
    fn init():
        pass

    virtual fn add_both(ref mut a: int, ref mut b: int):
        a += 1
        b += 1

class Doubler(Adder):
    fn init():
        super.init()

    override fn add_both(ref mut a: int, ref mut b: int):
        a += 2
        b += 2

fn main():
    x = 0
    f: fn(ref mut int, ref mut int) = add_both
    f(ref mut x, ref mut x)    #$ error[E3022]: `f(ref mut x, ref mut x)` names `x` twice, and `f` is a function value, which may hold any function, so no copy of it can be made for one variable; call the function by its name, `add_both(ref mut x, ref mut x)`, or give the second name a value of its own: `y = x` above the call and `f(ref mut x, ref mut y)`
    k: Adder = Doubler()
    k.add_both(ref mut x, ref mut x)    #$ error[E3022]
