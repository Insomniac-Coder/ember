#$ test: compile-fail
#$ rules: UNS-10
# `[UNS-10]` — an `UnsafeCell` is refused in `@no_alias_checks` code (`E3105`): such code proves its
# aliasing when compiling, and a cell leaves it to unsafe code.

from std.mem import UnsafeCell

@no_alias_checks
fn count(c: UnsafeCell[i32]) -> i32:    #$ error[E3105]: `UnsafeCell` is not permitted in `@no_alias_checks` code
    return 1

fn main():
    println(count(UnsafeCell(5)))
