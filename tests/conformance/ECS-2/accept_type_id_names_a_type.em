#$ test: run-pass
#$ rules: ECS-2, TYP-44
#$ profiles: debug, development
#$ stdout: true true
# `[ECS-2]` — the registry keys its pools by `TypeId`, "the value `std.core.type_id[T]()` gives":
# one type has one, two types never share one.
from std.core import type_id, TypeId

struct A:
    x: int

fn main():
    a: TypeId = type_id[A]()
    println(a == type_id[A](), a != type_id[int]())
