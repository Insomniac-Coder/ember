#$ test: compile-fail
#$ rules: UNS-10b, EFF-12

from std.mem import UnsafeCell

@no_alias_checks
fn forbidden():
    cell = UnsafeCell(1) #$ error[E3105]: `UnsafeCell` is not permitted in `@no_alias_checks` code
    println(cell.into_inner())

fn main():
    forbidden()
