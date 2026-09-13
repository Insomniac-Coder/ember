#$ test: compile-fail
#$ rules: ARN-8, PHIL-10, TST-23
# The representation field is compiler-private. No source-level field access
# can turn storage into initialized `T` without crossing `assume_init`.

fn main():
    slot: MaybeUninit[i32] = MaybeUninit[i32].uninit()
    println(slot.value) #$ error[E1020]: `value` is private to `MaybeUninit_i32`'s module
