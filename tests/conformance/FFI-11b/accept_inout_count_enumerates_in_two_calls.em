#$ test: run-pass
#$ rules: FFI-11b, FFI-11, FFI-10
#$ profiles: debug, development, release
#$ stdout: 4
#$ 2 1 4
#$ 4 1 4 9 16
#$ 0
# `[FFI-11b]` — the two-call enumeration idiom: `count: inout_count(items)`
# with `items: nullable count(count)`. The safe function takes
# `Option[MutSpan[T]]` and returns the count: `None` asks C how many there
# are (C gets a null pointer and a count of zero), `Some` hands C the span
# and its length as the capacity, and the call returns the number C wrote.
# A hand declaration writes the count at its C ABI position as `mut count`
# (C's `uint32_t*`), hidden from the callable signature as a `count(n)`
# witness is. The C side is the exported Ember function below.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(count, inout_count(items)), param(items, borrowed, count(count), nullable, exclusive), link_name="list_squares")
    safe fn squares(mut count: u32, items: Option[MutSpan[i32]])

pub extern "C" fn list_squares(count: *mut u32, items: *mut i32):
    total: u32 = 4
    capacity: u32 = 0
    unsafe:  # SAFETY(ffi): C's contract: `count` is valid, `items` holds `*count` elements or is null
        capacity = read(count, 0)
    if items.is_null():
        unsafe:  # SAFETY(ffi): `count` is valid
            write(count, 0, total)
        return
    written: u32 = 0
    while written < capacity and written < total:
        square = ((written + 1) * (written + 1)) as i32
        unsafe:  # SAFETY(ffi): `written` is below the capacity C was given
            write(items, written as usize, square)
        written += 1
    unsafe:  # SAFETY(ffi): `count` is valid
        write(count, 0, written)

fn main():
    total = squares(None)
    println(total)
    few: Array[i32] = [0, 0]
    println(squares(Some(few.as_mut_span())), few[0], few[1])
    all: Array[i32] = [0, 0, 0, 0]
    println(squares(Some(all.as_mut_span())), all[0], all[1], all[2], all[3])
    none: Array[i32] = Array[i32]()
    println(squares(Some(none.as_mut_span())))
