#$ test: compile-fail
#$ rules: UNS-10b, EFF-12

from std.mem import UnsafeCell

struct Wrapper:
    cell: UnsafeCell[i32]

@static_safe
fn forbidden(owned wrapper: Wrapper) -> i32: #$ error[E3105]: `UnsafeCell` is not permitted in `@static_safe` code
    return wrapper.cell.into_inner()

fn main():
    println(forbidden(Wrapper(UnsafeCell(1))))
