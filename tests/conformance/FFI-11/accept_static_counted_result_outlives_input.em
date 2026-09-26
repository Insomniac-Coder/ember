#$ test: run-pass
#$ rules: FFI-10, FFI-11, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ 41

unsafe extern "C":
    @ffi(param(input, borrowed, count(n)), result(borrowed, count(n), from(static)), link_name="copy_counted")
    safe fn copy(input: Span[i32], n: usize) -> Span[i32]

pub extern "C" fn copy_counted(input: *i32, n: usize) -> *i32:
    unsafe:
        result: *mut i32 = alloc[i32](n)
        write(result, 0, read(input, 0) + 1)
        return result as *i32

fn detached() -> Span[i32]:
    values: Array[i32] = Array[i32]()
    values.push(40)
    return copy(values.as_span())

fn main():
    result: Span[i32] = detached()
    println(result.len())
    println(result[0])
