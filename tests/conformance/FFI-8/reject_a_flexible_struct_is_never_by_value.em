#$ test: compile-fail
#$ rules: FFI-8
# `[FFI-8]` — a struct a flexible array member ends has no size of its own: it is used by value
# nowhere (`E5017`), only behind a reference or a pointer.

import c "packets.h"

fn copy_of(packet: ref c.Packet) -> c.Packet:    #$ error[E5017]: ends with a flexible array member
    return packet

fn main():
    pass
