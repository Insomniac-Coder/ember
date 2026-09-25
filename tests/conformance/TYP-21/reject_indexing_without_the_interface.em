#$ test: compile-fail
#$ rules: TYP-21, TYP-40, IFC-3
# `[TYP-40]` — a type is indexed only through `Index`: a method named `index`
# is not one. A type with `Index` and no `IndexMut` is read, not written; and
# `IndexMut` needs `Index` (`[IFC-3]`).

struct Bag:
    items: Array[int]

extend Bag:
    fn index(self, i: int) -> ref int:
        return ref self.items[i]

struct Ro:
    items: Array[int]

extend Ro implements Index[int]:
    type Output = int

    fn index(self, i: int) -> ref int:
        return ref self.items[i]

fn main():
    b = Bag(items=[1])
    println(b[0])    #$ error[E2020]: cannot index `Bag`
    r = Ro(items=[1])
    println(r[0])
    r[0] = 5    #$ error[E3021]: cannot write through a shared reference
