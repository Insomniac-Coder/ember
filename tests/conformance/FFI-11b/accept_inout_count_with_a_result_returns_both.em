#$ test: run-pass
#$ rules: FFI-11b
#$ profiles: debug, development, release
#$ stdout: 1 3
#$ 0 2 1 8
#$ 1 3 1 8 27
# `[FFI-11b]` — the two-call idiom on a C function that has a result of its own (Windows'
# `GetLogicalProcessorInformation` returns a `BOOL`): the Ember function returns a tuple, C's
# result first, then the count C wrote back. Here C returns 1 when every item fit and 0 when the
# span was too short, having filled what fit. The C side is the exported Ember function below.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(count, inout_count(items)), param(items, borrowed, count(count), nullable, exclusive), link_name="list_cubes")
    safe fn cubes(mut count: u32, items: Option[MutSpan[i32]]) -> i32

pub extern "C" fn list_cubes(count: *mut u32, items: *mut i32) -> i32:
    total: u32 = 3
    capacity: u32 = 0
    unsafe:  # SAFETY(ffi): C's contract: `count` is valid, `items` holds `*count` elements or is null
        capacity = read(count, 0)
    if items.is_null():
        unsafe:  # SAFETY(ffi): `count` is valid
            write(count, 0, total)
        return 1
    written: u32 = 0
    while written < capacity and written < total:
        cube = ((written + 1) * (written + 1) * (written + 1)) as i32
        unsafe:  # SAFETY(ffi): `written` is below the capacity C was given
            write(items, written as usize, cube)
        written += 1
    unsafe:  # SAFETY(ffi): `count` is valid
        write(count, 0, written)
    if written == total:
        return 1
    return 0

fn main():
    ok, total = cubes(None)
    println(ok, total)
    few: Array[i32] = [0, 0]
    short, written = cubes(Some(few.as_mut_span()))
    println(short, written, few[0], few[1])
    all: Array[i32] = [0, 0, 0]
    filled, count = cubes(Some(all.as_mut_span()))
    println(filled, count, all[0], all[1], all[2])
