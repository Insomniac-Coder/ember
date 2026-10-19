#$ test: run-pass
#$ rules: ECS-7, DRV-1
#$ profiles: debug, release
#$ stdout: 3
#$ Position false true
#$ Velocity true true
#$ Health false true
#$ Position
# `[ECS-7]` — "`@derive(Component)` registers the type in a compile-time component registry that
# serialisation and reflection-based tools read". `components()` is that registry: every type of
# the program that derives `Component`, in declaration order, with its name, identity and layout,
# written by the compiler; `Plain` derives other things only and is not in it.
from std.ecs import Component, components, fnv1a

@derive(Copy, Component)
struct Position:
    x: f32

@derive(Copy, Component)
@component(layout=soa)
struct Velocity:
    x: f32

@derive(Copy, Eq)
struct Plain:
    x: f32

@derive(Copy, Component)
enum Health:
    Alive
    Dead

fn main():
    found = components()
    println(found.len())
    for c in found:
        println(c.name, c.soa, c.hash == fnv1a(c.name))
    println(Position.component_name())
