# Clause EXC-16.1: "Giving a field that owns memory a new value (`h.items = []`, `h.name = other`) is
# a reshape: inside a `for` loop over the same field it is `E3020` (`[BRW-18]`), and a view into the
# old value is not used after it (`E3060`, `[BRW-17]`)."
#$ test: compile-fail
#$ rules: EXC-16, BRW-18, BRW-17, BRW-10
# The new value given directly and through a method (`reset`, by its summary). No text in the
# spec for these: code only.

class Hold:
    items: Array[int] = []
    name: String

    fn init(owns name: String):
        this.name = name

    fn reset():
        this.items = []

fn main():
    h = Hold(String("a"))
    h.items.push(owns 1)
    for x in h.items:
        println(x)
        h.items = []    #$ error[E3020]
    for x in h.items:
        println(x)
        h.reset()    #$ error[E3020]
    n = h.name
    other = String("b")
    h.name = other
    println(n)    #$ error[E3060]
