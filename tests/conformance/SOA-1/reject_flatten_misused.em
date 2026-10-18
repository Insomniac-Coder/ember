#$ test: compile-fail
#$ rules: SOA-1
# `[SOA-1]` — `@soa(flatten)` is the one form; it splits a field whose type is a struct into a
# column per field of its own, so such a field has no column of its own to name.
from std.math import Vec3

struct Body:
    @soa(flatten)
    pos: Vec3
    @soa(split)  #$ error[E0104]: `@soa` takes `flatten`
    vel: Vec3

struct Tagged:
    @soa(flatten)
    tag: int
    weight: f32

fn main():
    bodies = SoA[Body].with_capacity(2)
    whole = bodies.pos  #$ error[E2020]: `pos` is split into a column per field by `@soa(flatten)`
    tags = SoA[Tagged].with_capacity(2)  #$ error[E0104]: `@soa(flatten)` splits a struct field, and `tag` is a `i64`
