#$ test: run-pass
#$ rules: ERR-3, TYP-22
#$ profiles: debug, development, release
#$ stdout: load failed / Load(cause=Disk(code=7))
#$ load failed
#$ caused by: disk 7
#$ disk 9 Disk(code=9)
#$ disk 9
# `[ERR-3]` — `Error` is `Display + Debug` with `source` (default `None`). A
# type writes its `Display`, takes its field-wise `Debug`, and may write
# `source` in an `extend` block, where it replaces the default (D-603). A
# `ref dyn Error` and a `Box[dyn Error]` print through their table, `Debug`
# included for a type that writes none (D-602: the table's body for that is
# `std`'s, named by nothing else).

from std.fmt import Formatter, FmtError

struct Disk:
    code: int

extend Disk implements Display, Error:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"disk {self.code}")

struct Load:
    cause: Disk

extend Load implements Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("load failed")

extend Load implements Error:
    fn source(self) -> Option[ref dyn Error]:
        return Some(ref self.cause)

fn chain(e: ref dyn Error):
    println(f"{e} / {e!r}")
    println(e)
    next = e.source()
    while true:
        match next:
            Some(c):
                println(f"caused by: {c}")
                next = c.source()
            None:
                break

fn main():
    l = Load(Disk(7))
    chain(ref l)
    b: Box[dyn Error] = Box(Disk(9))
    println(f"{b} {b!r}")
    println(b)
