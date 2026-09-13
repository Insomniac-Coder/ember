#$ test: compile-fail
#$ rules: ARN-9, OWN-3, TST-23

fn main():
    arena = Arena.with_capacity(32)
    slots: MutSpan[MaybeUninit[i32]] = arena.alloc_uninit[i32](1)
    slots.write_at(0, 7)
    unsafe:
        values: MutSpan[i32] = slots.assume_init()
        slots.write_at(0, 8) #$ error[E3040]: `slots` has been moved out of
        println(values[0])
