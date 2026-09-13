#$ test: run-pass
#$ rules: UNS-10, UNS-10a
#$ assert-c: contains("&((")
#$ assert-c: !contains("refcell")
# `get` crosses exactly one unsafe boundary and returns a raw pointer. It does
# not expose a safe reference or add a runtime borrow-state mechanism.

from std.mem import UnsafeCell

fn change(shared: ref UnsafeCell[i32]):
    unsafe:
        pointer: *mut i32 = shared.get()
        write(pointer, 0, 42)

fn main():
    cell: UnsafeCell[i32] = UnsafeCell[i32](40)
    unsafe:
        pointer: *mut i32 = cell.get()
        write(pointer, 0, 41)
        println(read(pointer, 0))
    change(ref cell)
    println(cell.into_inner())
    # `self` is borrowed, so an expression-local temporary remains usable for
    # the enclosing statement. Keeping the raw pointer beyond that point would
    # remain the unsafe author's validity obligation under `[UNS-4]`.
    unsafe:
        println(read(UnsafeCell(43).get(), 0))
#$ stdout: 41
#$ 42
#$ 43
