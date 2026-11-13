#$ test: run-fail
#$ rules: UNS-10a
#$ panics: index 2 is out of bounds for a length of 2
# `[UNS-10a]` — `UnsafeCell` suspends no rule globally: an index read from a cell is bounds-checked
# as any index is.

from std.mem import UnsafeCell

fn main():
    xs: Array[int] = [10, 20]
    cell = UnsafeCell(1)
    # SAFETY(aliasing): nothing else reaches the cell's storage while the write runs
    unsafe:
        write(cell.get(), 0, 2)
    i = cell.into_inner() as usize
    println(xs[i])
