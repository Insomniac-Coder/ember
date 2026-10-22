#$ test: run-pass
#$ rules: ECS-4, ECS-5, THR-8, CLO-10
#$ profiles: debug, development, release
#$ stdout: true true
#$ stdout: [0, 1] [3, 5]
#$ stdout: 5 1 true
# `[ECS-4]` (ODR-218) — `run_parallel` runs systems whose sets do not conflict at once, on threads:
# `left` and `right` each say they have started and wait for the other, so each sees the other
# only when both run at the same time (run one after the other, the first would wait out its ten
# seconds alone). Each system's `Commands` are applied once all have returned, in the systems'
# order, so `left`'s marks come before `right`'s whatever order they ran in. A system may be a
# lambda: what it captures is `Send` (`[THR-8]`, `[CLO-10]`), here a `@sync` object.

import std.time
from std.ecs import World, Query, Mut, Component, Entity, system
from std.sync import Atomic

@derive(Copy, Component)
struct Pos:
    x: int

@derive(Copy, Component)
struct Vel:
    x: int

@derive(Copy, Component)
struct Mark:
    by: int

static LEFT: Atomic[int] = Atomic(0)
static RIGHT: Atomic[int] = Atomic(0)
static LEFT_SAW_RIGHT: Atomic[bool] = Atomic(false)
static RIGHT_SAW_LEFT: Atomic[bool] = Atomic(false)
static TWO: Atomic[int] = Atomic(0)

## Waits up to ten seconds for `flag` to be set by the other system.
fn other_started(flag: ref Atomic[int]) -> bool:
    start = time.Instant.now()
    while flag.load() == 0 and start.elapsed().as_millis() < 10000:
        time.sleep(time.Duration.from_millis(1))
    return flag.load() == 1

fn left(mut q: Query[(Entity, Pos)]):
    LEFT.store(1)
    LEFT_SAW_RIGHT.store(other_started(ref RIGHT))
    for e, p in q.iter():
        q.commands().add(e, Mark(p.x))

fn right(mut q: Query[(Entity, Mut[Vel])]):
    RIGHT.store(1)
    RIGHT_SAW_LEFT.store(other_started(ref LEFT))
    cmds = q.commands()
    for e, v in q.iter_mut():
        v.x += 1
        cmds.add(e, Mark(v.x))

@sync
class Tally:
    total: Atomic[int] = Atomic(0)

fn main():
    w = World()
    a = w.create()
    w.add(a, Pos(3))
    b = w.create()
    w.add(b, Vel(4))
    w.run_parallel([left, right])
    println(LEFT_SAW_RIGHT.load(), RIGHT_SAW_LEFT.load())
    marked = w.entities_with[Mark]()
    marks: Array[int] = []
    for e in marked:
        marks.push(e.index() as int)
    seen: Array[int] = []
    w.run(fn(mut q: Query[Mark], unused: ()): seen.extend([m.by for m in q.iter()]), ())
    println(marks, seen)
    tally = Tally()
    counted = system(owned fn(mut q: Query[Vel]): tally.total.fetch_add(q.iter().map(fn(v: ref Vel) => v.x).sum()), "counted")
    w.run_parallel([counted, system(owned fn(mut q: Query[Pos]): TWO.store(q.iter().count()), "two")])
    println(tally.total.load(), TWO.load(), w.has[Mark](a))
