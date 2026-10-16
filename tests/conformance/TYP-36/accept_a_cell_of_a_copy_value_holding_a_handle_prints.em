#$ test: run-pass
#$ rules: TYP-36, CELL-2, TYP-39
#$ profiles: debug, release
#$ stdout: Holder(held=Pin(left))
#$ stdout: Holder(held=Pin(right))
#$ stdout: right
# `[TYP-36]` (ODR-109) — a `Cell` inside another value shows a copy of what it holds; a `Copy` value
# holding a class handle, whose type writes its own `Debug`, is copied with the handle's retain and
# let go after, so its own `fmt_debug` may `set` the cell while it runs and the handle lives on.

from std.fmt import Formatter, FmtError

class Tag:
    name: String

@derive(Copy)
struct Pin:
    tag: Tag

extend Pin implements Debug:
    fn fmt_debug(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"Pin({self.tag.name})")

struct Holder:
    held: Cell[Pin]

fn main():
    h = Holder(held=Cell(Pin(tag=Tag(name="left"))))
    println(f"{h!r}")
    h.held.set(Pin(tag=Tag(name="right")))
    println(f"{h!r}")
    println(h.held.get().tag.name)
