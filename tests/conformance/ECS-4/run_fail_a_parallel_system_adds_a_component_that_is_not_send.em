#$ test: run-fail
#$ rules: ECS-4, ECS-5, THR-8
#$ profiles: debug, development
#$ panics: a system `run_parallel` runs adds a `Sprite`, which is not Send
# `[ECS-4]` (ODR-218), `[ECS-5]` — a component `run_parallel` uses is `Send`: a system it runs on
# another thread that asks its `Commands` to add one that is not (here a `Texture` handle, whose
# class is not `@sync`) would hand it to the calling thread, and panics before it does. The same
# addition from `world.run`, on the calling thread, is made.
from std.ecs import World, Query, Component, Entity

class Texture:
    id: int

    fn init(mut self, id: int):
        self.id = id

@derive(Component)
struct Sprite:
    texture: Texture

@derive(Copy, Component)
struct Pos:
    x: int

fn decorate(mut q: Query[(Entity, Pos)]):
    for e, p in q.iter():
        q.commands().add(e, Sprite(Texture(p.x)))

fn decorate_now(mut q: Query[(Entity, Pos)], unused: ()):
    for e, p in q.iter():
        q.commands().add(e, Sprite(Texture(p.x)))

fn main():
    w = World()
    e = w.create()
    w.add(e, Pos(1))
    w.run(decorate_now, ())
    println(w.has[Sprite](e))
    w.run_parallel([decorate])
