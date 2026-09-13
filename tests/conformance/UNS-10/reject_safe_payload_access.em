#$ test: compile-fail
#$ rules: UNS-10, MOD-2

from std.mem import UnsafeCell

fn main():
    cell = UnsafeCell(1)
    println(cell.value) #$ error[E1020]: private
