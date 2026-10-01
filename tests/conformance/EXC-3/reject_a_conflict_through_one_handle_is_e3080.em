#$ test: compile-fail
#$ rules: EXC-3, LT-5
# `[EXC-3]` — conflicting accesses to a class field through one local the
# compiler can see are rejected at compile time with `E3080` (shape X1), and
# `[LT-5]` names the place in source terms, `b.items`, never `b.0` (D-466).

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]

fn main():
    b = Bag()
    v = b.items[..]
    b.items.push(4)    #$ error[E3080]: `b.items` is borrowed here and mutably borrowed elsewhere
    println(v)
    w = b.items[..]
    b.items = [9]      #$ error[E3080]: `b.items` cannot be written while it is borrowed
    println(w)
