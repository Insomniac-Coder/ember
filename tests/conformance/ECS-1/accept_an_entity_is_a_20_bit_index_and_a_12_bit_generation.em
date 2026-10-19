#$ test: run-pass
#$ rules: ECS-1
#$ profiles: debug, release
#$ stdout: 0 0 true
#$ 1048575 4095 4294967295 true
#$ 5 3 3145733
#$ 0 1 false true
# `[ECS-1]` — "`Entity` is a 32-bit generational handle: a 20-bit index and a 12-bit generation …
# `Entity.NULL` is all ones." A destroyed entity's index comes back at its next generation, and the
# old handle is no longer alive.
from std.ecs import Entity, World

fn main():
    w = World()
    e = w.create()
    println(e.index(), e.generation(), w.is_alive(e))
    n = Entity.NULL
    println(n.index(), n.generation(), n.to_bits(), n.is_null())
    made = Entity.new(5u32, 3u32)
    println(made.index(), made.generation(), made.to_bits())
    w.destroy(e)
    again = w.create()
    println(again.index(), again.generation(), w.is_alive(e), w.is_alive(again))
