#$ test: compile-fail
#$ rules: BRW-11, BRW-1
# Two mutable arguments borrow the same place for one call. The conflict is
# simultaneous at call entry, even though the callee's parameter types are valid.

fn update_both(mut left: i32, mut right: i32):
    left = 10
    right = 20

fn main():
    value: i32 = 0
    update_both(value, value) #$ error[E3022]: `value` is already mutably borrowed
