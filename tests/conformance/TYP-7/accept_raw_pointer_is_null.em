#$ test: run-pass
#$ rules: TYP-7
#$ profiles: debug, release, shipping
#$ stdout: true

fn is_null_safely(pointer: *mut i32) -> bool:
    return pointer.is_null()

fn main():
    unsafe:
        pointer: *mut i32 = 0 as *mut i32
        println(is_null_safely(pointer))
