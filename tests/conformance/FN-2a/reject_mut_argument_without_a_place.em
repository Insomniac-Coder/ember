#$ test: compile-fail
#$ rules: FN-2a
#$ error[E3027]: a `mut` argument is not a mutable place
# A `mut` mode is read from the callee's signature. The compiler must form a
# mutable borrow at this boundary, so an unaddressable value is diagnostic B10
# rather than the generic `[EXP-5]` assignment-place error.

fn double(mut n: i32):
    n = n + n

fn main():
    double(1 + 1)
