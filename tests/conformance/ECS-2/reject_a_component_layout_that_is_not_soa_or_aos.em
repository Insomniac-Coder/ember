#$ test: compile-fail
#$ rules: ECS-2, ATT-1
#$ error[E0104]: `@component` takes `layout=soa` or `layout=aos`
#$ error[E0104]: only a struct is stored as an `SoA`
# `[ECS-2]` — a component's layout is `soa` or `aos` (the default), and only a struct can be an
# `SoA` (`[SOA-1]`).
from std.ecs import Component

@derive(Copy, Component)
@component(layout=columns)
struct Position:
    x: f32

@derive(Copy, Component)
@component(layout=soa)
enum Mode:
    Fast
    Slow

fn main():
    println(Position.component_name(), Mode.component_name())
