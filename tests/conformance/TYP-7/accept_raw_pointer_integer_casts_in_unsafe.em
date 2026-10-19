#$ test: run-pass
#$ rules: TYP-7, UNS-1
#$ profiles: debug, development, release
#$ stdout: true

fn main():
    bits: usize
    unsafe:  # SAFETY: the pointer is only cast, never read
        p: *mut i32 = 0 as *mut i32
        bits = p as usize
    println(bits == 0)
