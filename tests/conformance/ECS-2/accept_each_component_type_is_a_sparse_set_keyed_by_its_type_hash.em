#$ test: run-pass
#$ rules: ECS-2, SOA-1
#$ profiles: debug, development
#$ stdout: true true
#$ true false
#$ [Entity(bits=0), Entity(bits=2), Entity(bits=1)]
#$ [Entity(bits=0), Entity(bits=1)]
#$ [Entity(bits=1)]
#$ 2 2
# `[ECS-2]` — "Each component type is stored as a sparse set: a dense `Array[T]` or `SoA[T]`
# (`@component(layout=soa)`; the default is `aos`), the entities in dense order, and a sparse index
# by entity. A component type's identity is the 64-bit FNV-1a hash of its fully qualified name
# (`ecs.type_hash[T]()`)." Removing an entity's component moves the last one into its slot; a
# destroyed entity leaves every storage. `count` names the `aos` storage's class in this module.
from std.ecs import Entity, World, Component, AosStorage, type_hash, fnv1a
from std.math import Vec3

@derive(Copy, Component)
struct Position:
    value: Vec3

@derive(Copy, Component)
@component(layout=soa)
struct Velocity:
    value: Vec3

fn count(s: AosStorage[Position]) -> int:
    return s.dense.len()

fn main():
    println(type_hash[Position]() == fnv1a("Position"), type_hash[Velocity]() != type_hash[Position]())
    println(Velocity.component_soa(), Position.component_soa())
    w = World()
    a = w.create()
    b = w.create()
    c = w.create()
    w.add(a, Position(Vec3.ZERO))
    w.add(b, Position(Vec3.ZERO))
    w.add(c, Position(Vec3.ZERO))
    w.add(a, Velocity(Vec3.ZERO))
    w.add(b, Velocity(Vec3.ZERO))
    w.remove[Position](b)
    w.add(b, Position(Vec3(1.0, 0.0, 0.0)))
    w.remove[Position](b)
    w.add(b, Position(Vec3(2.0, 0.0, 0.0)))
    println(w.entities_with[Position]())
    println(w.entities_with[Velocity]())
    w.destroy(a)
    println(w.entities_with[Velocity]())
    match w.storage[Position]():
        Some(any):
            positions = cast any as AosStorage[Position]
            println(count(positions), w.len())
        None:
            println("none")
