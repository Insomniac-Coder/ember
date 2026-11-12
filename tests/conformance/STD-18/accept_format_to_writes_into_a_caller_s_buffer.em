#$ test: run-pass
#$ rules: STD-18, EFF-1, TYP-26
#$ profiles: debug, development, release
#$ stdout: n = 42 ratio 0.5! 17
#$ stdout: refused: does not fit
#$ stdout: (1, 2) [1, 2, 3]
#$ stdout: exactly
#$ stdout: [7]: 1.0
# `[STD-18]` — `format_to(ref mut buf, …, sep=" ", end="")` writes its values' text, as `print`
# does, into the caller's buffer and gives a view of it; `Err(FmtError.Failed)` when it does not fit.
# Its own machinery never allocates (`[EFF-1]`), so a `@noalloc` function may call it; a written
# `Display` writes through its `fmt`, which may call `format_to` itself. A buffer filled exactly
# fits; an empty one fits only nothing.

from std.fmt import format_to, Formatter, FmtError

struct Point:
    x: int
    y: int

extend Point implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"({this.x}, {this.y})")

struct Cell:
    id: int
    value: float

extend Cell implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        bytes = [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8]
        match format_to(ref mut bytes.as_mut_span(), "[", this.id, "]", sep = ""):
            Ok(label):
                return f.write_str(f"{label}: {this.value}")
            Err(error):
                return Err(error)

@noalloc
fn label(ref mut buf: MutSpan[u8], n: int, ratio: float) -> Result[StrView, FmtError] from buf:
    return format_to(ref mut buf, "n =", n, "ratio", ratio, end = "!")

fn zeros(n: int) -> Array[u8]:
    bytes: Array[u8] = Array[u8]()
    for _ in range(n):
        bytes.push(owns 0)
    return bytes

fn main():
    bytes = zeros(32)
    match label(ref mut bytes.as_mut_span(), 42, 0.5):
        Ok(text):
            println(text, text.len())
        Err(_):
            println("refused")
    small = zeros(4)
    match format_to(ref mut small.as_mut_span(), "does not fit"):
        Ok(text):
            println(text)
        Err(_):
            println("refused: does not fit")
    match format_to(ref mut bytes.as_mut_span(), Point(x = 1, y = 2), [1, 2, 3]):
        Ok(text):
            println(text)
        Err(_):
            println("refused")
    seven = zeros(7)
    match format_to(ref mut seven.as_mut_span(), "exactly"):
        Ok(text):
            println(text)
        Err(_):
            println("refused")
    empty = zeros(0)
    match format_to(ref mut empty.as_mut_span(), ""):
        Ok(text):
            match format_to(ref mut bytes.as_mut_span(), Cell(id = 7, value = 1.0), text, sep = ""):
                Ok(shown):
                    println(shown)
                Err(_):
                    println("refused")
        Err(_):
            println("refused")
