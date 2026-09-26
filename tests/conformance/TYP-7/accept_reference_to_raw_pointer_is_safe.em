#$ test: run-pass
#$ rules: TYP-7, UNS-1
#$ profiles: debug, release, shipping
#$ stdout: 7

fn main():
    value: i32 = 7
    pointer: *i32 = (ref value) as *i32
    unsafe:
        println(read(pointer, 0))
