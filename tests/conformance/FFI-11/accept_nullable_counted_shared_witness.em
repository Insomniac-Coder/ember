#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ 0
#$ 2

unsafe extern "C":
    @ffi(param(left, borrowed, count(n), nullable), param(right, borrowed, count(n), nullable), link_name="shared_optional_count")
    safe fn compare(left: Option[Span[i32]], right: Option[Span[i32]], n: usize) -> usize

pub extern "C" fn shared_optional_count(left: *i32, right: *i32, n: usize) -> usize:
    return n

fn main():
    empty: Array[i32] = Array[i32]()
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    println(compare(None, None))
    println(compare(None, Some(empty.as_span())))
    println(compare(Some(values.as_span()), Some(values.as_span())))
