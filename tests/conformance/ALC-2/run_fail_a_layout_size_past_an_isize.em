#$ test: run-fail
#$ rules: ALC-2
#$ panics: `invalid Layout`: a size that rounded up to a multiple of its alignment does not fit in an `isize`
## `[ALC-2]` — `Layout(size, align)` checks that `size` rounded up to a multiple of `align` fits in an
## `isize`, and panics (`invalid Layout`) otherwise: 9,223,372,036,854,775,801 rounds up to 2^63.

from std.mem import Layout

fn main():
    largest = Layout(9223372036854775800, 8)
    println(largest.size)
    l = Layout(9223372036854775801, 8)
    println(l.size)
