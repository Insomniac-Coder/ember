#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 2
#$ 31

unsafe extern "C":
    @ffi(param(input, borrowed, count(n)), result(borrowed, count(n), exclusive, from(static)), link_name="fresh_mut_span")
    safe fn fresh(input: Span[i32], n: usize) -> MutSpan[i32]

pub extern "C" fn fresh_mut_span(input: *i32, n: usize) -> *mut i32:
    data: *mut i32
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        data = alloc[i32](n)
        write(data, 0, 31)
    return data

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    result: MutSpan[i32] = fresh(values.as_span())
    println(result.len())
    println(result[0])
