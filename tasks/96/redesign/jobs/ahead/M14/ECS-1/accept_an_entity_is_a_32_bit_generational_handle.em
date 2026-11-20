# Clause ECS-1.1: "`Entity` is a 32-bit generational handle: a 20-bit index and a 12-bit generation
# (`Handle` with `Split20`, `[HND-2]`), EnTT's default `entt::entity`; `Entity.NULL` is all ones
# (`entt::null`)."
#$ test: run-pass
#$ rules: ECS-1, ECS-3, HND-2
#$ stdout: 4
#$ stdout: 1 false
#$ stdout: true 1
#$ stdout: false
# An entity's size, its generation after a destroy (`[ECS-3]`'s `destroy` gives the next one), the
# index reused with the new generation, and `Entity.NULL`, which no registry holds.

from std.ecs import Registry, Entity

fn main():
    println(size_of[Entity]())
    reg = Registry()
    e = reg.create()
    g = reg.destroy(e)
    println(g, reg.valid(e))
    e2 = reg.create()
    println(reg.valid(e2), reg.current(e))
    println(reg.valid(Entity.NULL))
