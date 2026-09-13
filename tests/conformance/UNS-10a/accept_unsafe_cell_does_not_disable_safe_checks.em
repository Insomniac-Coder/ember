#$ test: run-pass
#$ rules: UNS-10, UNS-10a

from std.mem import UnsafeCell

fn main():
    cell = UnsafeCell(1)
    unsafe:
        pointer = cell.get()
        write(pointer, 0, 2)
    # Leaving the unsafe block restores the ordinary safe boundary; consuming
    # extraction remains safe because no shared cell survives it.
    println(cell.into_inner())
#$ stdout: 2
