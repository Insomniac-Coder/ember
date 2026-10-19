#$ test: run-pass
#$ rules: TYP-36, TYP-39, LEX-19, STD-9, TYP-17
#$ profiles: debug, development, release
#$ stdout: thing 3
#$ a thing 3 b Thing(v=3)
#$ <thing 3> <5>
#$ red custom teal
#$ (1 & 2) (thing 1 & thing 2)
#$ node#7 <node#7>
#$ thing 9 [thing 9]
#$ [thing 3] thing 3
#$ 21 C
#$ [thing 3] [  thing 3]
#$ [Thing(v=5), Thing(v=4)] Some(Thing(v=3))
#$ thing 1, thing 2
# `[TYP-36]` — a struct, an enum or a class has `Display` by implementing it:
# its `fmt` writes through the `Formatter` it is given (`std.fmt`), and
# `f"{x}"`, `print(x)`, `to_string` and a `T: Display` bound use that text. A
# `Box` and a `Cell` show as what they hold (`[TYP-39]`); inside a collection,
# an `Option` or `{x!r}` a value shows its `Debug`, field-wise here. `{x!s}`
# is the written text, which a spec pads as text.

from std.fmt import Formatter, FmtError

struct Thing implements Display:
    v: int

    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"thing {self.v}")

enum Color:
    Red
    Custom(name: String)

extend Color implements Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        match self:
            Color.Red:
                return f.write_str("red")
            Color.Custom(name):
                f.write_str("custom ")?
                return f.write_str(name)

struct Pair[T]:
    a: T
    b: T

extend[T: Display] Pair[T] implements Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"({self.a} & {self.b})")

class Node implements Display:
    id: int

    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"node#{self.id}")

@derive(Copy)
struct Temp implements Display:
    c: int

    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"{self.c} C")

fn show[T: Display](x: T) -> String:
    return f"<{x}>"

fn by_ref(t: ref Thing) -> String:
    return f"[{t}]"

fn main():
    t = Thing(3)
    println(t)
    println(f"a {t} b {t!r}")
    println(show(t), show(5))
    println(Color.Red, Color.Custom(String.from("teal")))
    println(Pair[int](1, 2), f"{Pair[Thing](Thing(1), Thing(2))}")
    n = Node(7)
    println(n, show(n))
    b = Box(Thing(9))
    println(b, f"[{b}]")
    println(by_ref(ref t), t.to_string())
    println(Cell(Temp(21)))
    println(f"[{t!s}] [{t!s:>9}]")
    println([Thing(5), Thing(4)], Some(t))
    println([Thing(1), Thing(2)].join(", "))
