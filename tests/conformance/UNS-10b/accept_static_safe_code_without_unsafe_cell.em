#$ test: run-pass
#$ rules: UNS-10b, EFF-12

from std.mem import UnsafeCell

@static_safe
fn add_one(value: i32) -> i32:
    return value + 1

fn main():
    # The module may import and ordinary code may use the expert primitive;
    # the prohibition is scoped to the `@static_safe` function itself.
    cell: UnsafeCell[i32] = UnsafeCell(41)
    println(add_one(cell.into_inner()))
#$ stdout: 42
