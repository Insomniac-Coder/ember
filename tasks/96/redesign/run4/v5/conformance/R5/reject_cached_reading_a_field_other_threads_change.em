#$ test: compile-fail
#$ rules: R5, R15
#$ help: remove `@cached` (line 18)
## R5, R15: a cached method may not read a field other threads change under a lock: a kept result could
## be stale (AN-21, area E's ruling, E2117; completeness G1).

import std.thread
from std.sync import Lock

class Mesh:
    lock: Lock = Lock()
    verts: Array[f64] guarded by lock = []

    fn add(v: f64):
        with this.lock:
            this.verts.push(v)

    @cached
    fn radius() -> f64:    #$ error[E2117]: `radius` reads `verts`, which other threads change under `lock`; a kept result could be stale; remove `@cached` (line 18)
        with this.lock:
            r = 0.0
            for v in this.verts:
                r = max(r, abs(v))
            return r

fn main():
    m = thread_shared Mesh()
    t = thread.spawn(fn() thread_shared m: m.add(2.0))
    t.join()
    println(m.radius())
