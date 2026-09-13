#$ test: compile-fail
#$ rules: UNS-10b, EFF-12

from std.mem import UnsafeCell

struct Wrapper:
    marker: i32

    @static_safe
    fn forbidden(self):
        cell = UnsafeCell(1) #$ error[E3105]: `UnsafeCell` is not permitted in `@static_safe` code
        println(cell.into_inner())

fn main():
    Wrapper(0).forbidden()
