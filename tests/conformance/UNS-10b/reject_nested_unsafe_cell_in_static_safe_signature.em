#$ test: compile-fail
#$ rules: UNS-10b, EFF-12

from std.mem import UnsafeCell

struct Wrapper:
    cell: UnsafeCell[i32]

@no_alias_checks
fn forbidden(owns wrapper: Wrapper) -> i32: #$ error[E3105]: `UnsafeCell` is not permitted in `@no_alias_checks` code
    return wrapper.cell.into_inner()

fn main():
    println(forbidden(Wrapper(UnsafeCell(1))))
