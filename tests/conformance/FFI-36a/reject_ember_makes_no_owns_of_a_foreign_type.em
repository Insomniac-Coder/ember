#$ test: compile-fail
#$ rules: FFI-36a, TYP-45
# `[FFI-36a]` (N175) — an `owns T` of a foreign type holds only an object foreign code made, so Ember
# makes none: `owns` before a value of such a struct made in Ember is `E2114`, naming where to get one
# and, for a sized struct, where to keep this one.

@layout(c)
struct Packet:
    len: i32

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("`p` is a packet C made, not freed yet")
    unsafe fn packet_free(p: *mut Packet)
    @ffi(result(returns_owned(destructor=packet_free), one))
    safe fn packet_new(len: i32) -> owns Packet

fn take(owns p: owns Packet):
    pass

fn main():
    take(owns Packet(len=0))    #$ error[E2114]: `Packet`'s objects are made by C and ended by `packet_free` (line 14), so an `owns Packet` holds only one C made; get one from `packet_new`, or keep this one where it is made: `Packet(len=0)`
