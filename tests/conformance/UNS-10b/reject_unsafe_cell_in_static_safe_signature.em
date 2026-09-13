#$ test: compile-fail
#$ rules: UNS-10b, EFF-12

from std.mem import UnsafeCell

@static_safe
fn forbidden(owned cell: UnsafeCell[i32]) -> i32: #$ error[E3105]: `UnsafeCell` is not permitted in `@static_safe` code
    return cell.into_inner()

fn main():
    println(forbidden(UnsafeCell(1)))
