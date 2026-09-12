#$ test: compile-fail
#$ rules: LT-4, ARN-1
# The returned mutable reference carries the region of the shared arena borrow;
# a local arena ends with this frame and therefore cannot source the return.

fn escaped() -> ref mut i32:
    arena = Arena.with_capacity(64)
    return arena.alloc(1)          #$ error[E3061]: arena allocation cannot outlive `arena`

fn main():
    println(0)
