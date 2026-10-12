#$ test: run-pass
#$ rules: TYP-7
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ 11

fn main():
    value: i32 = 7
    shared: *i32 = ref_to_ptr(ref value)
    unsafe:  # SAFETY: this test sets up what the operation requires
        println(read(shared, 0))
    other: i32 = 2
    mutable: *mut i32 = ref_to_ptr(ref mut other)
    unsafe:  # SAFETY: this test sets up what the operation requires
        write(mutable, 0, 11)
    println(other)
