#$ test: compile-fail
#$ rules: UNS-10, TYP-15a

from std.mem import UnsafeCell

fn main():
    values: Array[i32] = Array[i32]()
    view = values.as_span()
    cell = UnsafeCell(view) #$ error[E3063]: `Span[i32]` is a view, so it may not be stored in an UnsafeCell's contents
    println(cell)
