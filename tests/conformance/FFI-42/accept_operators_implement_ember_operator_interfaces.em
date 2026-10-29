#$ test: run-pass
#$ rules: FFI-42
#$ warning[L3015]: `Vec2.sub` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Grid.new` is a C++ function the program calls in unsafe code
#$ warning[W5002]: `Vec2.operator!=` is not imported from `operators.hpp`: `operator!=`: Ember's `!=` is `Eq`'s
#$ warning[W5002]: `Vec2.operator<` is not imported from `operators.hpp`: `operator<`: Ember orders through `Ord.cmp`
#$ profiles: debug, development, release
#$ stdout: 11 22 -11 -22
#$ stdout: false true true
#$ stdout: 12 24
#$ stdout: 3 6
#$ stdout: a negative scale
#$ stdout: 9 0
#$ stdout: -10 -20
#$ stdout: 16
#$ stdout: 350 450 true
#$ stdout: 12
# `[FFI-42]` — C++ overloaded operators map to Ember's operator interfaces: `operator+` is `Add`,
# unary `operator-` `Neg`, `operator+=` `AddAssign`, `operator==` `Eq` (so `!=` too), a throwing
# `operator*` `Mul` whose `Output` is the `Result` its thunk gives, and the `const` and other
# `operator[]` `Index` and `IndexMut`, so `a + b`, `-a`, `a += b`, `a == b`, `g[i]` and `g[i] = v`
# call them; an operator declared outside its class is its first operand's class's (`bank.Money`),
# as is one a class defines as its friend (`bank.Points`).
# An operator an overlay entry does not assert safe to call (`operator-` with an operand, `sub`)
# is an `unsafe` method of its class, under its interface method's name.

from std.ffi import CppError, ForeignBox
import cpp "operators.hpp" with (overlay = "overlays/operators.em")

fn difference(a: cpp.Vec2, b: cpp.Vec2) -> cpp.Vec2:
    unsafe:  # SAFETY(ffi): `sub` takes and gives values; its overlay asserts nothing
        return a.sub(b)

fn new_grid() -> Result[ForeignBox[cpp.Grid], CppError]:
    unsafe:  # SAFETY(ffi): the constructor makes a grid the box owns
        return cpp.Grid.new()

fn main():
    a = cpp.Vec2(x = 1, y = 2)
    b = cpp.Vec2(x = 10, y = 20)
    c = a + b
    d = -c
    println(c.x, c.y, d.x, d.y)
    println(a == b, a != b, c == a + b)
    e = c
    e += a
    println(e.x, e.y)
    match a * 3:
        Ok(f):
            println(f.x, f.y)
        Err(error):
            println(error.what)
    match a * -1:
        Ok(f):
            println(f.x, f.y)
        Err(error):
            println(error.what)
    match new_grid():
        Ok(cells):
            cells.get_mut()[0] = 9
            seven = cells.get()[0] - 2
            cells.get_mut()[3] = seven
            println(cells.get()[0], cells.get()[1])
        Err(error):
            println(error.what)
    g = difference(a, c)
    println(g.x, g.y)
    match new_grid():
        Ok(cells):
            cells.get_mut()[2] = 16
            println(cells.get().total())
        Err(error):
            println(error.what)
    price = cpp.bank.Money(cents = 100)
    total = price + cpp.bank.Money(cents = 250)
    paid = total
    paid += price
    println(total.cents, paid.cents, total == price + cpp.bank.Money(cents = 250))
    scaled = cpp.bank.Points(value = 4) * 3
    println(scaled.value)
