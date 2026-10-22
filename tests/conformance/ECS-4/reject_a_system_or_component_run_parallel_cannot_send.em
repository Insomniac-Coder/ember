#$ test: compile-fail
#$ rules: ECS-4, THR-8, THR-9, CLO-10
# `[ECS-4]` (ODR-218, ODR-255) — `run_parallel` runs each system on another thread, so what a system
# captures and every component its query reads or writes must be `Send` (`E7004`, naming it, where
# the system is made: here in the literal, or by `system`), and a component two systems of one
# run read must be `Sync` (`E7005`, naming both systems). A `Texture` handle is neither (its class
# is not `@sync`, `[THR-1]`); a `Cell` is `Send` but not `Sync`.
from std.ecs import World, Query, Mut, Component, system

class Texture:
    id: int

    fn init(mut self, id: int):
        self.id = id

@derive(Component)
struct Sprite:
    texture: Texture

@derive(Component)
struct Gauge:
    level: Cell[int]

@derive(Copy, Component)
struct Pos:
    x: int

fn draw(mut q: Query[Sprite]):
    pass

fn read_gauge(mut q: Query[Gauge]):
    pass

fn read_gauge_too(mut q: Query[(Gauge, Pos)]):
    pass

fn main():
    w = World()
    w.run_parallel([draw])  #$ error[E7004]: the system `draw`'s query uses `Sprite`, which is not Send
    w.run_parallel([read_gauge, read_gauge_too])  #$ error[E7005]: systems `read_gauge` and `read_gauge_too` cannot run at once: both read `Gauge`, which is not Sync
    shared = Texture(1)
    by_value = system(owned fn(mut q: Query[Pos]): println(shared.id), "by_value")  #$ error[E7004]: the system `by_value` captures `shared`, a `Texture`, which is not Send
    w.run_parallel([by_value])
