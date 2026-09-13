#$ test: run-pass
#$ rules: ARN-8, ARN-8a, TST-23
# `MaybeUninit[T]` is Copy exactly when `T` is. Copies are independent
# storage; initializing one does not initialize or alias the other.

fn main():
    first: MaybeUninit[i64] = MaybeUninit[i64].uninit()
    second = first
    first.write(10i64)
    second.write(20i64)
    unsafe:
        a: i64 = first.assume_init()
        b: i64 = second.assume_init()
        println(a)
        println(b)
#$ stdout: 10
#$ 20
