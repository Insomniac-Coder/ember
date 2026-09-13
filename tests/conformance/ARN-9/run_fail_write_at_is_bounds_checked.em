#$ test: run-fail
#$ rules: ARN-9, SPN-2, TST-23
#$ panics: index 1 is out of bounds for a length of 1

fn main():
    arena = Arena.with_capacity(16)
    slots: MutSpan[MaybeUninit[i32]] = arena.alloc_uninit[i32](1)
    slots.write_at(1, 7)
