#$ test: run-pass
#$ rules: SPN-8, SPN-9, BRW-2, TST-25

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    view = values.as_mut_span()
    pointer: *mut i32 = view.as_mut_ptr()
    view[0] = 2
    unsafe:
        write(pointer, 0, 3)
    println(view[0])
    _detached: *i32 = values.as_span().as_ptr()
    values.push(4)
    println(values.len())
#$ stdout: 3
#$ stdout: 2
