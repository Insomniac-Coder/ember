#$ test: run-pass
#$ rules: ARN-9, ARN-13, TST-23

fn main():
    arena = Arena.with_capacity(64)
    slots: MutSpan[MaybeUninit[i32]] = arena.alloc_uninit[i32](3)
    first: ref mut i32 = slots.write_at(0, owns 10)
    println(first)
    slots.write_at(1, owns 20)
    slots.write_at(2, owns 30)
    values: MutSpan[i32]
    unsafe:  # SAFETY(uninit): every slot is written before it is read
        values = slots.assume_init()
    println(values[1])
    println(values[2])
#$ stdout: 10
#$ 20
#$ 30
