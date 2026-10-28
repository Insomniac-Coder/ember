#$ test: run-pass
#$ rules: FFI-8, FFI-12
#$ profiles: debug, development, release
#$ stdout: 3 103
#$ stdout: 4
#$ stdout: 3.5
#$ stdout: 7
#$ stdout: 10
#$ stdout: 12 12
# `[FFI-8]` (ODR-249) — `@ffi(newtype)` on a `typedef` makes a nominal type with the typedef's
# representation: `fd_t` is not `c_int`, `seconds_t` not `f64`, `widget_ref` not the pointer, and
# each converts to and from its representation with `as` only (a literal takes the representation's
# type). It is passed to C as the typedef is: `later` takes and returns `seconds_t` where C has a
# `double`, in the register a `double` goes in. A typedef the overlay does not name stays an alias:
# `count_t` is `c_int`. A hand declaration writes the same: `@ffi(newtype) type T = R`, `Copy` as
# its representation.

from std.ffi import c_int

import c "./newtypes.c" with (overlay = "overlays/newtypes.em") as nt

@ffi(newtype)
type ticket_t = c_int

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(link_name="twice")
    safe fn twice_ticket(n: ticket_t) -> ticket_t

fn main():
    fd = nt.open_slot(3)
    println(nt.slot_number(fd), fd as c_int)
    made = 104 as nt.fd_t
    copy = made
    println(nt.slot_number(copy))
    start = 1.5 as nt.seconds_t
    println(nt.later(start, 2.0) as f64)
    unsafe:  # SAFETY(ffi): the widget is made, read and freed once, here
        widget = nt.make_widget(7)
        println(nt.widget_size(widget))
        nt.free_widget(widget)
    plain: c_int = nt.twice(5)
    println(plain)
    ticket = twice_ticket(6 as ticket_t)
    println(ticket as c_int, nt.twice(6))
