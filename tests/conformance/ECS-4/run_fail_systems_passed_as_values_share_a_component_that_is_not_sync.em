#$ test: run-fail
#$ rules: ECS-4, THR-9
#$ profiles: debug, development
#$ stdout: before
#$ panics: systems `first` and `second` cannot run at once: both read `Gauge`, which is not Sync
# `[ECS-4]` (ODR-255) — systems passed as values are checked against each other before any of them
# starts: two that read one component read it from two threads at once, so it must be `Sync`, and
# a `Cell` is not. The run panics, naming both systems and the component.
from std.ecs import World, Query, Component, System

@derive(Component)
struct Gauge:
    level: Cell[int]

fn first(mut q: Query[Gauge]):
    println("first ran")

fn second(mut q: Query[Gauge]):
    println("second ran")

fn main():
    w = World()
    e = w.create()
    w.add(e, Gauge(Cell(1)))
    systems: Array[System] = [first, second]
    println("before")
    w.run_parallel(systems)
