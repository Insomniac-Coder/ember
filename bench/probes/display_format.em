## Prices ADR-186's written `Display`: five million points, each formatted into
## a new `String` (`f"{p}"`, whose `fmt` writes `(x, y)`), summing the
## lengths. Twin: `display_format.c` (`snprintf` into a fresh buffer).

from std.fmt import Formatter, FmtError

struct Point implements Display:
    x: int
    y: int

    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"({self.x}, {self.y})")

fn main():
    total = 0
    for i in range(5000000):
        p = Point(i, -i)
        s = f"{p}"
        total += s.len()
    println(total)
