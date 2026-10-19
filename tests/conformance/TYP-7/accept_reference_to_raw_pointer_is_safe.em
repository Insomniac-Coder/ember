#$ test: run-pass
#$ rules: TYP-7, UNS-1
#$ profiles: debug, development, release
#$ stdout: 7

fn main():
    value: i32 = 7
    pointer: *i32 = (ref value) as *i32
    unsafe:  # SAFETY: this test sets up what the operation requires
        println(read(pointer, 0))
