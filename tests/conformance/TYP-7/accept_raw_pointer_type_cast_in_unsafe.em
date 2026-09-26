#$ test: run-pass
#$ rules: TYP-7, UNS-1
#$ profiles: debug, release, shipping
#$ stdout: 7

fn main():
    unsafe:
        value: *mut i32 = alloc[i32](1)
        bytes: *mut u8 = value as *mut u8
        restored: *mut i32 = bytes as *mut i32
        write(restored, 0, 7)
        println(read(value, 0))
        free(value, 1)
