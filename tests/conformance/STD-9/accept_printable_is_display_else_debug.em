#$ test: run-pass
#$ rules: STD-9, TYP-47
#$ profiles: debug, development, release
#$ stdout: log: 1 two Pos(x=3, y=4) named 5 [6, 7] Some(8)
#$ Pos(x=1, y=2) & named 3
# `[STD-9]` (verdict 117) — the compiler implements `Printable` for every type that has `Display`
# or `Debug`: a value's text is its `Display` text when its type has `Display`, its `Debug` text
# otherwise. A program's own function takes any number of printable values as `print` does
# (`[Ts...: needs Printable]` above `fn log(values: Ts...):`), and `fmt_print` writes a printable
# value's text through a `Formatter`.

from std.fmt import Formatter, FmtError

@derive(Debug)
struct Pos:
    x: int
    y: int

struct Named:
    n: int

extend Named implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"named {this.n}")

struct Both:
    a: Pos
    b: Named

extend Both implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        this.a.fmt_print(ref mut f)?
        f.write_str(" & ")?
        return this.b.fmt_print(ref mut f)

[Ts...: needs Printable]
fn log(values: Ts...):
    print("log:")
    for v in values:
        print("", v)
    println()

fn main():
    o: Option[int] = Some(8)
    log(1, "two", Pos(3, 4), Named(5), [6, 7], o)
    println(Both(Pos(1, 2), Named(3)))
