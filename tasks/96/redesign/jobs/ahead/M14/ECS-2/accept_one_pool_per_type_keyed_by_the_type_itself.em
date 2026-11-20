# Clause ECS-2.1: "Each component type is stored in a pool of its own, a sparse set (...); removal is
# swap-and-pop"; "The registry keys its pools by the type itself (...) `TypeId` the value
# `std.core.type_id[T]()` gives (...), which two types of one program never share, whatever their
# names"; "`reg.storage[T]()` reaches its own pool as a place"; "`ecs.type_hash[T]()`, the 64-bit
# FNV-1a hash of `std.core.type_name[T]()` (...), is stable across builds"
#$ test: run-pass
#$ rules: ECS-2, ECS-3, TYP-44
#$ stdout: true true
#$ stdout: false false
#$ stdout: 3 7
# A type's id and hash are the same at every use and differ between two types; the storage walked
# as a place changes its values. The hash's value and `type_name`'s text depend on the package's
# name and version: not pinned here.

from std.ecs import Registry
import std.ecs as ecs

struct Position:
    x: int

struct Velocity:
    x: int

fn main():
    println(std.core.type_id[Position]() == std.core.type_id[Position](), ecs.type_hash[Position]() == ecs.type_hash[Position]())
    println(std.core.type_id[Position]() == std.core.type_id[Velocity](), ecs.type_hash[Position]() == ecs.type_hash[Velocity]())
    reg = Registry()
    a = reg.create()
    b = reg.create()
    reg.emplace(a, owns Position(x=1))
    reg.emplace(b, owns Position(x=5))
    for e, ref mut p in reg.storage[Position]():
        p.x += 2
    println(reg.get[Position](a).x, reg.get[Position](b).x)
