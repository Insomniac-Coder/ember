#$ test: run-pass
#$ rules: SOA-4, ARN-5
#$ profiles: debug, release
#$ stdout: 0 3 true
#$ true true true false
#$ 3 3
#$ 1.0 2.0 3.0
#$ 6.5
#$ 2.0 20
#$ 2
#$ true
# `[SOA-4]` — `ArenaSoA[T]` is the fixed-capacity, arena-backed `SoA`, under the rules of `[ARN-5]`:
# its storage is taken from the arena once, at construction; it never grows or moves; adding past
# its capacity returns `Err(CapacityError.Full)` and changes nothing. Its columns and elements are
# an `SoA`'s.
from std.collections import ArenaSoA, CapacityError

@derive(Copy)
struct Mote:
    x: f32
    tag: i32

fn added(r: Result[void, CapacityError]) -> bool:
    match r:
        Ok(_done):
            return true
        Err(_full):
            return false

fn sum(xs: Span[f32]) -> f32:
    total: f32 = 0.0
    for x in xs:
        total += x
    return total

fn main():
    arena = Arena.with_capacity(4096)
    motes = ArenaSoA[Mote].with_capacity(arena, 3)
    println(motes.len(), motes.capacity(), motes.is_empty())
    a = added(motes.push(Mote(x=1.0, tag=10)))
    b = added(motes.push(Mote(x=2.0, tag=20)))
    c = added(motes.push(Mote(x=3.0, tag=30)))
    d = added(motes.push(Mote(x=4.0, tag=40)))
    println(a, b, c, d)
    println(motes.len(), motes.capacity())
    println(motes.x[0], motes.x[1], motes.x[2])
    motes[2].x += 0.5
    println(sum(motes.x))
    second = motes.get(1).unwrap()
    println(second.x, second.tag)
    gone = motes.swap_remove(0)
    println(motes.len())
    println(gone.tag == 10)
