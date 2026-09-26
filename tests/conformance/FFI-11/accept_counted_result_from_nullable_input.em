#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ 0
#$ 2
#$ 52

unsafe extern "C":
    @ffi(param(input, borrowed, count(n), nullable), result(borrowed, count(n), from(static)), link_name="fresh_at_optional_count")
    safe fn fresh(input: Option[Span[i32]], n: usize) -> Span[i32]

pub extern "C" fn fresh_at_optional_count(input: *i32, n: usize) -> *i32:
    unsafe:
        data: *mut i32 = alloc[i32](2)
        write(data, 0, 52)
        write(data, 1, 53)
        return data as *i32

fn main():
    empty: Array[i32] = Array[i32]()
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    println(fresh(None).len())
    println(fresh(Some(empty.as_span())).len())
    result: Span[i32] = fresh(Some(values.as_span()))
    println(result.len())
    println(result[0])
