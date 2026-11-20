# Clause ECS-8.2: "`.connect(owns f)` keeps `f: fn(ref mut Registry, Entity)` in the registry, so `f`
# is given with `owns` (`[FN-2a]`) and what it holds goes with it (a listener that views a local the
# registry outlives is `E3063`)"
#$ test: compile-fail
#$ rules: ECS-8, CLO-4, LT-1
# `name` ends with `install`; the registry, the caller's, keeps the listener. No text in the spec:
# code only. `[ECS-8]` says `E3063`; `[CLO-4]` and `[LT-42]` say `E3026` for a function value that
# captures a view with `ref` and outlives it: Q1. This file follows `[ECS-8]`.

from std.ecs import Registry, Entity

@derive(Copy)
struct Hp:
    n: int

fn install(ref mut reg: Registry):
    name = String("hp")
    reg.on_construct[Hp]().connect(owns fn(ref mut r, ref e) ref name: println(name, r.valid(e)))    #$ error[E3063]

fn main():
    reg = Registry()
    install(ref mut reg)
    reg.emplace(reg.create(), owns Hp(n=1))
