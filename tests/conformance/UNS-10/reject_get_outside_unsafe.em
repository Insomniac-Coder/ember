#$ test: compile-fail
#$ rules: UNS-10, UNS-1

from std.mem import UnsafeCell

fn main():
    cell = UnsafeCell(1)
    _pointer = cell.get() #$ error[E3100]: `UnsafeCell.get` needs an `unsafe` block
