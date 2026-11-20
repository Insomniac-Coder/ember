# Clause BRW-11.1: "Two arguments of one call (the object a method is called on counts as one), of
# which at least one may be changed (a `ref mut` argument, a `MutSpan` one included; the object of a
# method that may change it; or a function value, which counts as changing every place the
# functions it may run change and every real variable they capture, `[CLO-2]`), may not name one
# place, and neither may reach into what the other owns, unless the compiler proves they differ";
# "Any other possible overlap is refused when compiling: `E3022` where both may be changed, `E3021`
# otherwise."
#$ test: compile-fail
#$ rules: BRW-11, BRW-3, BCK-5, CLO-17, EFF-26
# One case per kind of changeable argument. `f(ref mut v, v[0])` is `[BRW-3]`'s own refused call.
# The spec gives no text for these: code only.

static mut LOG: Array[String] = []

fn wipe():
    LOG.clear()

fn f(ref mut v: Array[String], s: StrView):
    v.push(owns String(s))

fn show_then(s: StrView, g: fn()):
    println(s)
    g()

fn both(ref mut n: int, owns g: fn()):
    n += 1
    g()

fn pair(ref mut a: int, ref mut b: int):
    a += b

struct Words:
    names: Array[String]

    fn add_copy(s: StrView):
        this.names.push(owns String(s))

fn main():
    v = [String("a")]
    f(ref mut v, v[0])    #$ error[E3021]
    w = Words([String("b")])
    w.add_copy(w.names[0])    #$ error[E3021]
    LOG.push(owns String("c"))
    show_then(LOG[0], wipe)    #$ error[E3021]
    count = 0
    both(ref mut count, owns fn(): count += 1)    #$ error[E3022]
    xs = [1, 2, 3]
    i = 0
    j = 1
    pair(ref mut xs[i], ref mut xs[j])    #$ error[E3022]
