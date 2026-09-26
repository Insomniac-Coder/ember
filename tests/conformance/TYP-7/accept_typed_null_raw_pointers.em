#$ test: run-pass
#$ rules: TYP-7
#$ profiles: debug, release, shipping
#$ stdout: true
#$ true

fn main():
    shared: *i32 = null[*i32]()
    mutable: *mut i32 = null[*mut i32]()
    println(shared.is_null())
    println(mutable.is_null())
