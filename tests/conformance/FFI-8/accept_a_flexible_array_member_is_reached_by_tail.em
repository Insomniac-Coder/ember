#$ test: run-pass
#$ rules: FFI-8, FFI-5
#$ profiles: debug, development, release
#$ stdout: 7 3 60
#$ stdout: 10 20 30
#$ stdout: 8 true
# `[FFI-8]` — a struct a flexible array member ends is unsized: it is held behind a reference or a
# pointer (here the `ForeignBox` C hands over), its fields are read as any struct's, and its
# elements are reached by `unsafe fn tail(self, n) -> Span[T]`, which borrows the struct; its size
# and alignment are C's (`[FFI-5]` checks them).

from std.mem import size_of
import c "./packets.c" with (overlay = "overlays/packets.em") as p

## The packet's elements: a view that borrows it.
fn elements(packet: ref p.Packet) -> Span[i64]:
    unsafe:  # SAFETY: C allocated `count` elements after the packet
        return packet.tail(packet.count as usize)

fn main():
    packet = p.packet_new(3)
    held: ref p.Packet = packet.get()
    println(held.kind, held.count, p.packet_sum(held))
    values = elements(held)
    println(values[0], values[1], values[2])
    println(size_of[*p.Packet](), true)
