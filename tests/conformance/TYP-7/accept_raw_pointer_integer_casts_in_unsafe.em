#$ test: run-pass
#$ rules: TYP-7, UNS-1
#$ profiles: debug, release, shipping
#$ stdout: true

fn main():
    unsafe:
        p: *mut i32 = 0 as *mut i32
        bits: usize = p as usize
        println(bits == 0)
