#$ test: compile-fail
#$ rules: FFI-36a, FFI-11
# `[FFI-36a]` (N175) — the foreign types are an `extern type`, an opaque C++ class, a `@ffi(handle)` type
# and a `@layout(c)` struct that a contract gives a destructor. A `returns_owned` result giving an
# `owns T` of any other type is `E5002`, since C cannot make an object whose block only Ember lays out:
# an Ember struct with no C layout, or a scalar.

@layout(c)
struct Packet:
    len: i32

struct Counter:
    n: i32

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("`p` is a packet C made, not freed yet")
    unsafe fn packet_free(p: *mut Packet)
    @safety("`c` is a counter C made, not freed yet")
    unsafe fn free_counter(c: *mut Counter)
    @ffi(result(returns_owned(destructor=packet_free), one))
    safe fn packet_new(len: i32) -> owns Packet
    @ffi(result(returns_owned(destructor=free_counter), one))
    safe fn make_counter() -> owns Counter    #$ error[E5002]: `make_counter` gives an `owns Counter` C made, and Ember makes the blocks of a `Counter`; declare a type for it, `type Counter` with `@ffi(destroys(free_counter))`, and write `-> owns Counter`
    @ffi(result(returns_owned(destructor=free_counter), one))
    safe fn make_int() -> owns i32    #$ error[E5002]: `make_int` gives an `owns i32` C made, and Ember makes the blocks of an `i32`; declare a type for it, `type Int` with `@ffi(destroys(free_counter))`, and write `-> owns Int`

fn main():
    pass
