#$ test: compile-fail
#$ rules: FFI-36a, OWN-8, TYP-45
# `[FFI-36a]` (N175) — an `owns T` of a foreign type is not `Clone`, since a copy would be a block Ember
# made that the type's destructor would end: `copy of` it is `E2116`.

@layout(c)
struct Packet:
    len: i32

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("`p` is a packet C made, not freed yet")
    unsafe fn packet_free(p: *mut Packet)
    @ffi(result(returns_owned(destructor=packet_free), one))
    safe fn packet_new(len: i32) -> owns Packet

fn main():
    p = packet_new(1)
    r = copy of p    #$ error[E2116]: `p` is an `owns Packet`, an object C made and `packet_free` ends, so `copy of` cannot copy it
    println(p.len, r.len)
