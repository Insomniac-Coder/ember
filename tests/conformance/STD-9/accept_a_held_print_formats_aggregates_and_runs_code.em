#$ test: run-pass
#$ rules: STD-9, TYP-39
#$ profiles: debug, development, release
#$ stdout: Point(x=1, y=2) [1, 2] Some(3) (4, 5) seven
#$ stdout: inner
#$ stdout: Loud 9 Point(x=3, y=4)
# `[STD-9]` (D-900) — a print statement holds its stream from its first piece to its last: an
# aggregate's `Debug`, a list, an `Option`, a tuple and a written `Display` are each formatted
# into their piece, and an argument whose evaluation prints (`noisy()`) prints before the statement
# takes the stream (the LLVM backend's port of the print pieces, tried against the C backend's).

from std.fmt import Formatter, FmtError

@derive(Debug)
struct Point:
    x: int
    y: int

struct Loud:
    n: int

extend Loud implements Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"Loud {self.n}")

fn noisy() -> int:
    println("inner")
    return 9

fn main():
    p = Point(1, 2)
    xs = [1, 2]
    o: Option[int] = Some(3)
    println(p, xs, o, (4, 5), "seven")
    q = Point(3, 4)
    println(Loud(noisy()), q)
