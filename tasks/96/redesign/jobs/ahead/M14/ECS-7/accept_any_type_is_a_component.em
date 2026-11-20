# Clause ECS-7.1: "Any type is a component, with no registration and no derive."
#$ test: run-pass
#$ rules: ECS-7, ECS-3
#$ stdout: 5
#$ stdout: ann
#$ stdout: 2
#$ stdout: 1
# A number, a `String`, an enum and a struct with no derive, each a component as it is.

from std.ecs import Registry

enum Mood:
    Calm
    Angry

struct Plain:
    a: int

fn main():
    reg = Registry()
    e = reg.create()
    reg.emplace(e, owns 5)
    reg.emplace(e, owns String("ann"))
    reg.emplace(e, owns Mood.Angry)
    reg.emplace(e, owns Plain(a=1))
    println(reg.get[int](e))
    println(reg.get[String](e))
    match reg.get[Mood](e):
        Mood.Calm:
            println(1)
        Mood.Angry:
            println(2)
    println(reg.get[Plain](e).a)
