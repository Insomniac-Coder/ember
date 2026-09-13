#$ test: compile-fail
#$ rules: UNS-10a, BRW-4

from std.mem import UnsafeCell

fn main():
    cell = UnsafeCell(1)
    value = 2
    unsafe:
        _pointer = cell.get()
        first: ref mut i32 = ref mut value
        second: ref mut i32 = ref mut value #$ error[E3022]: already mutably borrowed
        println(first)
        println(second)
