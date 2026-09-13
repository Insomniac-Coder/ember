#$ test: compile-fail
#$ rules: SPN-8, UNS-1, TST-25

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    pointer: *i32 = values.as_span().as_ptr()
    println(read(pointer, 0)) #$ error[E3100]: `read` needs an `unsafe` block

