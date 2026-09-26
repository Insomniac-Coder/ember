#$ test: run-pass
#$ rules: TYP-7, TYP-16
#$ profiles: debug, release, shipping
#$ stdout: true

fn empty[T]() -> *mut T:
    return null[*mut T]()

fn main():
    pointer: *mut i32 = empty[i32]()
    println(pointer.is_null())
