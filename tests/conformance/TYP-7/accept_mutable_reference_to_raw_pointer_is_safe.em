#$ test: run-pass
#$ rules: TYP-7, UNS-1
#$ profiles: debug, development, release
#$ stdout: 7

fn main():
    value: i32 = 1
    pointer: *mut i32 = (ref mut value) as *mut i32
    unsafe:  # SAFETY: this test sets up what the operation requires
        write(pointer, 0, 7)
    println(value)
