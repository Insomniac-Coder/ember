#$ test: compile-fail
#$ rules: EFF-13, EFF-12, EFF-9

# `[EFF-13]` — a `@static_safe` function keeps no dynamic aliasing check: a
# `mut self` call through a handle the compiler cannot prove alone (two
# receivers that may be one object, in a loop) is checked at run time
# (`[EXC-3]`), and so is a `RefCell` borrow (`[EFF-9]`): `E4030`, shape S1, at
# each.

class Bag:
    items: Array[int]

    fn add(mut self, x: int):
        self.items.push(x)

@static_safe
fn fill(mut b: Bag, mut other: Bag):
    for i in 0..3:
        b.add(i)    #$ error[E4030]: `@static_safe` function `fill` keeps an exclusivity check on a class object here
        other.add(i)    #$ error[E4030]: `@static_safe` function `fill` keeps an exclusivity check on a class object here

@static_safe
fn peek(cell: RefCell[int]) -> int:
    with _g = cell.borrow():    #$ error[E4030]: `@static_safe` function `peek` keeps a `RefCell` borrow check here
        return 1

fn main():
    b = Bag([])
    c = b
    fill(b, c)
    # A loop over the field holds a read of it, so a write to a `Bag` can
    # conflict somewhere: the checks in `fill` cannot be removed (`[EXC-3]`).
    for x in c.items:
        println(x)
