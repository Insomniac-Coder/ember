#$ test: run-pass
#$ rules: SPN-8, SPN-9, UNS-1, TST-25
#$ assert-c: contains("const int32_t*")

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    values.push(8)
    shared = values.as_span()
    shared_pointer: *i32 = shared.as_ptr()
    println(shared.len())

    mutable = values.as_mut_span()
    read_pointer: *i32 = mutable.as_ptr()
    write_pointer: *mut i32 = mutable.as_mut_ptr()
    mutable[0] = 9
    unsafe:
        println(read(shared_pointer, 0))
        println(read(read_pointer, 1))
        write(write_pointer, 1, 10)
    println(mutable[0])
    println(mutable[1])
#$ stdout: 2
#$ stdout: 9
#$ stdout: 8
#$ stdout: 9
#$ stdout: 10

