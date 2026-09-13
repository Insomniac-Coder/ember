#$ test: compile-fail
#$ rules: ARN-1, ARN-7, ARN-9, TST-23

fn main():
    arena = Arena.with_capacity(16)
    slots: MutSpan[MaybeUninit[i32]] = arena.alloc_uninit[i32](1)
    arena.reset() #$ error[E3021]: `arena` is borrowed here and mutably borrowed elsewhere
    slots.write_at(0, 1)
