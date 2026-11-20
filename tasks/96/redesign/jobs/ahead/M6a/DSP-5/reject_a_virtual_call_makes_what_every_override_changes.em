# Clause DSP-5.2: "What a call through a `virtual` or interface method may change, read and end is
# worked out from every override and implementation the program has (`[EFF-1]`)."
#$ test: compile-fail
#$ rules: DSP-5, EFF-1, EFF-24, BRW-18
# `s.grow()` may run `Square.grow`, which pushes to `LOG`, whatever `s` is when the program runs; so
# may `grow_one(s)`, through its body. Through `dyn` and interface values it needs M11: not here. No
# text in the spec for this program: code only.

static mut LOG: Array[String] = []

open class Shape:
    fn init():
        pass

    virtual fn grow():
        pass

class Square(Shape):
    fn init():
        super.init()

    override fn grow():
        LOG.push(owns String("square"))

fn grow_one(s: Shape):
    s.grow()

fn tick_all(s: Shape):
    for line in LOG:
        println(line)
        s.grow()    #$ error[E3020]
        grow_one(s)    #$ error[E3020]

fn main():
    LOG.push(owns String("start"))
    tick_all(Square())
