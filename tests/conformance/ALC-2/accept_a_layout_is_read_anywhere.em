#$ test: run-pass
#$ rules: ALC-2, MOD-2
#$ stdout: 24 8 0 1 65536
## `[ALC-2]` — `Layout` holds `size: usize` and `align: usize`, both `pub(read)`: read anywhere.
## `Layout(size, align)` accepts an alignment that is a power of two and a size that rounded up to
## it fits in an `isize`, the size 0 included.

from std.mem import Layout, PIECE

fn main():
    l = Layout(24, 8)
    empty = Layout(0, 1)
    println(l.size, l.align, empty.size, empty.align, PIECE)
