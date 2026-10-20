#$ test: run-pass
#$ rules: FFI-8, FFI-5
#$ profiles: debug, development, release
#$ stdout: 1 2 3 4 5.0
#$ stdout: 69 20 40
# `[FFI-8]` — an anonymous member is a field `_anon<N>` of a generated type `<Parent>_anon<N>`, and
# its fields are the parent's: `p.x` reads and writes `p._anon0.x`, through an anonymous member of
# an anonymous member too (`p.depth`). The layout is C's (`[FFI-5]`), so C reads what Ember wrote.

from std.ffi import c_int
import c "anonymous.h"

fn main():
    depth = c.Packet_anon0_anon0(depth = 0)
    p = c.Packet(kind = 0, _anon0 = c.Packet_anon0(x = 0, y = 0, _anon0 = depth), weight = 0.0)
    unsafe:  # SAFETY(ffi): the functions take and return values
        p = c.packet_make()
    println(p.kind, p.x, p.y, p.depth, p.weight)
    p.x = 20
    p.depth = 40
    sum: c_int = 0
    unsafe:  # SAFETY(ffi): as above
        sum = c.packet_sum(p)
    println(sum, p._anon0.x, p._anon0._anon0.depth)
