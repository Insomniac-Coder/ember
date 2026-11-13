#$ test: compile-fail
#$ rules: UNS-10a
# `[UNS-10a]` — `UnsafeCell` suspends no rule globally: in a function that writes through one, a view
# of a local returned past the local's end is still `E3060` (region checking), and a cell is still no
# `String` (type checking).

from std.mem import UnsafeCell

fn keep() -> Span[int]:
    xs: Array[int] = [1, 2]
    cell = UnsafeCell(3)
    # SAFETY(aliasing): nothing else reaches the cell's storage while the write runs
    unsafe:
        write(cell.get(), 0, 4)
    return xs[..]    #$ error[E3060]: `xs` does not live long enough

fn main():
    cell = UnsafeCell(1)
    s: String = cell    #$ error[E2020]: expected `String`, found `std.mem.UnsafeCell[i64]`
    println(keep(), s)
