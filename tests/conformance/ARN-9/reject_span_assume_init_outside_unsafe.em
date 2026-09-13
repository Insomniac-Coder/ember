#$ test: compile-fail
#$ rules: ARN-9, ARN-13, UNS-1, TST-23
# Only one of two slots is initialized. Safe conversion is rejected; spelling
# the call in `unsafe` would assert (not prove) the caller's full-initialization
# precondition and therefore is deliberately not executed by this test.

fn main():
    arena = Arena.with_capacity(16)
    slots: MutSpan[MaybeUninit[i32]] = arena.alloc_uninit[i32](2)
    slots.write_at(0, 7)
    values = slots.assume_init() #$ error[E3100]: `assume_init` needs an `unsafe` block
    println(values[0])
