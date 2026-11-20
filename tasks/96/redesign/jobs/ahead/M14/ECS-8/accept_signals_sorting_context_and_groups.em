# Clause ECS-8.1: "`on_construct[T]()`, `on_update[T]()` and `on_destroy[T]()` give a signal;
# `.connect(owns f)` keeps `f: fn(ref mut Registry, Entity)` in the registry (...) and
# `.disconnect(f)` compares and keeps nothing; the registry calls a listener after the change is
# made and before the method's result view is made, when no view of a pool is in use, so a listener
# that changes the registry is safe"; "`sort[T](less)` and `sort_as[To, From]()` reorder a pool.
# `ctx()` keeps one value per type (`emplace`, `get`, `get_mut`, `find`, `contains`, `erase`).
# `group[A, B]()` packs the owned pools in one order."
#$ test: run-pass
#$ rules: ECS-8, ECS-3, CLO-10, BRW-21
#$ stdout: 2
#$ stdout: 7
#$ stdout: 1 3
#$ stdout: 9
#$ stdout: false
#$ stdout: 1
# A listener that adds a component when one is made (and so changes the registry), disconnected by
# comparing it; the `emplace` result found again after it ran; a pool sorted; a context value; a
# group of two types.

from std.ecs import Registry, Entity

@derive(Copy)
struct Hp:
    n: int

@derive(Copy)
struct Tag:
    n: int

@derive(Copy)
struct Score:
    n: int

fn give_tag(ref mut reg: Registry, e: Entity):
    reg.emplace(e, owns Tag(n=1))

fn main():
    reg = Registry()
    reg.on_construct[Hp]().connect(owns give_tag)
    a = reg.create()
    h = reg.emplace(a, owns Hp(n=2))
    println(h.n)
    reg.on_construct[Hp]().disconnect(give_tag)
    b = reg.create()
    reg.emplace(b, owns Hp(n=7))
    println(reg.get[Hp](b).n)
    reg.sort[Hp](owns fn(x, y) => x.n < y.n)
    out: Array[int] = []
    for e, x in reg.view[Hp]():
        out.push(owns x.n)
    reg.emplace(b, owns Tag(n=3))
    println(reg.get[Tag](a).n, reg.get[Tag](b).n)
    reg.ctx().emplace(owns Score(n=9))
    println(reg.ctx().get[Score]().n)
    reg.ctx().erase[Score]()
    println(reg.ctx().contains[Score]())
    g = 0
    for e, x, t in reg.group[Hp, Tag]():
        g += t.n - 2
    println(g + out.len() - 1)
