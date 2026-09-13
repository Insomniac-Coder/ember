#$ test: run-pass
#$ rules: ARN-8, ARN-8a, ARN-13, TST-23
# A safe write establishes an initialized payload and returns a reference to
# it. The only read-out transition is the consuming unsafe assertion.

fn main():
    slot: MaybeUninit[i32] = MaybeUninit[i32].uninit()
    written: ref mut i32 = slot.write(41)
    println(written)
    unsafe:
        value: i32 = slot.assume_init()
        println(value + 1)
#$ stdout: 41
#$ 42
