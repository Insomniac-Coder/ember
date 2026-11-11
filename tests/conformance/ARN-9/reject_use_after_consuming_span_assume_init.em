#$ test: compile-fail
#$ rules: ARN-9, OWN-3, TST-23

fn main():
    arena = Arena.with_capacity(32)
    slots: MutSpan[MaybeUninit[i32]] = arena.alloc_uninit[i32](1)
    slots.write_at(0, owns 7)
    unsafe:  # SAFETY(uninit): every slot is written before it is read    #$ warning[L3010]: larger than necessary
        values: MutSpan[i32] = slots.assume_init()
        slots.write_at(0, owns 8) #$ error[E3040]: `slots` was handed over on line 9 and is used on line 10; to keep both, hand over a copy of your own: `copy of slots`
        println(values[0])
