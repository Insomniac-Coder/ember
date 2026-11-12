#$ test: compile-fail
#$ rules: STD-9, TYP-47
# `[STD-9]` (verdict 117) — `Printable` is the compiler's: no program writes it by hand (`E2040`),
# and a type with neither `Display` nor `Debug` does not have it, which a function over printable
# values refuses as `print` does (`E2040`, whose help is to implement `Display`).

from std.fmt import Formatter, FmtError

struct Plain:
    n: int

extend Plain implements Printable:    #$ error[E2040]: `Printable` is the compiler's
    fn fmt_print(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("plain")

[Ts...: needs Printable]
fn log(values: Ts...):
    for v in values:
        print(v)

fn main():
    log(1, RefCell(2))    #$ error[E2040]: `RefCell[i64]` implements neither `Display` nor `Debug`, which printing it needs
