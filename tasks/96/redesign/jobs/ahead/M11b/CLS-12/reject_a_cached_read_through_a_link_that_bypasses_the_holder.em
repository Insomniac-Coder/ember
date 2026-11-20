# Clause CLS-12.3: "A `@cached` body reads nothing a link may change without passing through the
# holder: reading a field of a class object reached through a field or element of `this` (not
# `this` itself), of a class that carries a weak count (`[OBJ-1]`), is `E2117`"
#$ test: compile-fail
#$ rules: CLS-12, EFF-27, OBJ-1, WK-11
# The spec's message up to its cross-references: the claim stops before "made from any name",
# whose `(`[WK-11]`)` may not be part of the message. `Book` carries a weak count, since the program
# makes a `weakref Book`. Claimed for the file, as the other `E2117`s (Q2).
#$ error[E2117]: `first_name` reads `items[0].name`, and a `Book` may be changed through a `weakref Book` link

class Book:
    name: String

    fn init(owns name: String):
        this.name = name

class Shelf:
    items: Array[Book] = []

    @cached
    fn first_name() -> String:
        return copy of this.items[0].name

fn main():
    s = Shelf()
    s.items.push(owns Book(String("a")))
    w = weakref s.items[0]
    println(s.first_name())
    if w exists as ref mut b:
        b.name = String("b")
