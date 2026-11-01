#$ test: compile-fail
#$ rules: UNS-10b, EFF-12

from std.mem import UnsafeCell

@no_alias_checks
fn forbidden(owns cell: UnsafeCell[i32]) -> i32: #$ error[E3105]: `UnsafeCell` is not permitted in `@no_alias_checks` code
    return cell.into_inner()

fn main():
    println(forbidden(owns UnsafeCell(1)))
