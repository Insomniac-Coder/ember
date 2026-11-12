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
    safe fn make_counter() -> owns Counter
    @ffi(result(returns_owned(destructor=free_counter), one))
    safe fn make_int() -> owns i32

fn take(owns p: owns Packet):
    pass

fn main():
    p = packet_new(1)
    take(owns Packet(len=0))
    r = copy of p
    println(p.len, r.len)
