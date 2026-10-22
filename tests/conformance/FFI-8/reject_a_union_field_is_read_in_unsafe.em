#$ test: compile-fail
#$ rules: FFI-8
# `[FFI-8]` — reading a C union's field needs `unsafe` (which field holds the value is not known),
# through an anonymous union member too; a union is made with exactly one field, by name.

import c "unions.h"

fn main():
    n = c.Number(i = 3)
    x = n.f    #$ error[E3100]: reading `f` of the union `c.h_unions.Number` requires an `unsafe` block
    v = c.Value(kind = 1, _anon0 = c.Value_anon0(real = 2.0))
    y = v.real    #$ error[E3100]: reading `real` of the union
    z = c.Number(i = 1, f = 2.0)    #$ error[E2020]: the union `c.h_unions.Number` is made with one of its fields, by name

@layout(c, union)
struct Owning:    #$ error[E0104]: the union's field `name` is a `String`, which is not `Copy`
    count: i32
    name: String
