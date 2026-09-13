#$ test: compile-fail
#$ rules: ARN-8, ARN-13, UNS-1, TST-23

fn main():
    slot: MaybeUninit[i32] = MaybeUninit[i32].uninit()
    slot.write(1)
    value = slot.assume_init() #$ error[E3100]: `assume_init` needs an `unsafe` block
    println(value)
