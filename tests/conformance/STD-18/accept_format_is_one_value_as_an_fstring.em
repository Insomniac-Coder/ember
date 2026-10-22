#$ test: run-pass
#$ rules: STD-18, MOD-5, LEX-19
#$ profiles: debug, development, release
#$ stdout: 3.14 42 0xff    hi [1, 2] (1, 2)
#$ stdout: 3.14159|3.14159
# `[STD-18]` — `format(value, spec="")`, the prelude's (`[MOD-5]`), is `f"{value:spec}"`: the
# value's text as the spec asks (`[LEX-19]`), its plain text without one, a written `Display`
# through its `fmt`.

from std.fmt import Formatter, FmtError

struct Point:
    x: int
    y: int

extend Point implements Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"({self.x}, {self.y})")

fn main():
    pi = 3.14159
    shown: String = format(pi, ".2f")
    println(shown, format(42), format(255, "#x"), format("hi", ">5"), format([1, 2]), format(Point(x = 1, y = 2)))
    println(format(pi, spec = ""), format(pi), sep = "|")
