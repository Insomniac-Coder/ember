#$ test: run-fail
#$ rules: ERR-4
#$ profiles: debug, development, release
#$ panics: opening: no such file
# `[ERR-4]` — `expect(msg)` on an `Err` panics with `<msg>: <Display text>`, the error's own `Display`.

from std.fmt import Formatter, FmtError

struct Missing:
    name: StrView

extend Missing implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("no such file")

fn main():
    r: Result[int, Missing] = Err(Missing(name="a"))
    println(r.expect("opening"))
