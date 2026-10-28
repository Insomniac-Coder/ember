#$ test: run-pass
#$ rules: FFI-11b, FFI-11, FFI-16
#$ profiles: debug, development, release
#$ stdout: 3 9
#$ stdout: 2 9 1 4
#$ stdout: ok 3
#$ stdout: ok 2 1 4
#$ stdout: busy
#$ stdout: 3 last 7
#$ stdout: 1 last 7 1
#$ stdout: hello 3
# `[FFI-11b]` — the two-call idiom beside the other outputs: the count is an output in its C
# position, so a function with `out` parameters returns them and the count in C's order, after
# C's own result when it has one, and a function returning a status returns `Result[T, N]`, `T`
# its outputs, count included (`[FFI-16]`). A C result with a contract of its own (a `cstr`
# from `static`) comes first as what its contract makes it. The C side is the exported Ember
# functions below.

@repr(i32)
@non_exhaustive
@ffi(status=Status, ok=Done, error=StatusError)
enum Status:
    Done = 0
    Busy = 1

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    # C: `void squares_with_max(uint32_t* count, int32_t* items, int32_t* max)`.
    @ffi(param(count, inout_count(items)), param(items, borrowed, count(count), nullable, exclusive), param(max, out, one), link_name="squares_with_max_c")
    safe fn squares_with_max(mut count: u32, items: Option[MutSpan[i32]], mut max: i32)

    # C: `int32_t list_squares(int32_t busy, uint32_t* count, int32_t* items)`, a status.
    @ffi(param(count, inout_count(items)), param(items, borrowed, count(count), nullable, exclusive), result(status), link_name="list_squares_c")
    safe fn list_squares(busy: i32, mut count: u32, items: Option[MutSpan[i32]]) -> Status

    # C: `const char* tagged(int32_t* tag, uint32_t* count, int32_t* items)`.
    @ffi(param(tag, out, one), param(count, inout_count(items)), param(items, borrowed, count(count), nullable, exclusive), result(borrowed, nul_terminated, from(static)), link_name="tagged_c")
    safe fn tagged(mut tag: i32, mut count: u32, items: Option[MutSpan[i32]]) -> cstr

fn fill(count: *mut u32, items: *mut i32) -> u32:
    total: u32 = 3
    capacity: u32 = 0
    unsafe:  # SAFETY(ffi): C's contract: `count` is valid, `items` holds `*count` elements or is null
        capacity = read(count, 0)
    if items.is_null():
        unsafe:  # SAFETY(ffi): `count` is valid
            write(count, 0, total)
        return total
    written: u32 = 0
    while written < capacity and written < total:
        unsafe:  # SAFETY(ffi): `written` is below the capacity C was given
            write(items, written as usize, ((written + 1) * (written + 1)) as i32)
        written += 1
    unsafe:  # SAFETY(ffi): `count` is valid
        write(count, 0, written)
    return written

pub extern "C" fn squares_with_max_c(count: *mut u32, items: *mut i32, max: *mut i32):
    fill(count, items)
    unsafe:  # SAFETY(ffi): C's contract: `max` is valid
        write(max, 0, 9)

pub extern "C" fn list_squares_c(busy: i32, count: *mut u32, items: *mut i32) -> i32:
    if busy != 0:
        return 1
    fill(count, items)
    return 0

pub extern "C" fn tagged_c(tag: *mut i32, count: *mut u32, items: *mut i32) -> cstr:
    fill(count, items)
    unsafe:  # SAFETY(ffi): C's contract: `tag` is valid
        write(tag, 0, 7)
    return c"last"

fn main():
    total, max = squares_with_max(None)
    println(total, max)
    two: Array[i32] = [0, 0]
    written, top = squares_with_max(Some(two.as_mut_span()))
    println(written, top, two[0], two[1])
    match list_squares(0, None):
        Ok(n):
            println("ok", n)
        Err(_):
            println("busy")
    pair: Array[i32] = [0, 0]
    match list_squares(0, Some(pair.as_mut_span())):
        Ok(n):
            println("ok", n, pair[0], pair[1])
        Err(_):
            println("busy")
    match list_squares(1, None):
        Ok(n):
            println("ok", n)
        Err(_):
            println("busy")
    word, tag, count = tagged(None)
    println(count, word.to_str().unwrap_or("?"), tag)
    one: Array[i32] = [0]
    word2, tag2, count2 = tagged(Some(one.as_mut_span()))
    println(count2, word2.to_str().unwrap_or("?"), tag2, one[0])
    println("hello", count)
