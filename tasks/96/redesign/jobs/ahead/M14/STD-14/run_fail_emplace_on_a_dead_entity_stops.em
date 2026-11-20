# Clause STD-14.1: "a constructor's or a mutation's caller's bug (`stepped(0)`, `BitSet.insert` of a
# negative, `Entity.new` out of range, `Registry.emplace` on a dead entity, (...)) panics"
#$ test: run-fail
#$ rules: STD-14, ECS-3, ERR-13
# The spec gives no stop text: the file claims the stop only.

from std.ecs import Registry

@derive(Copy)
struct Hp:
    n: int

fn main():
    reg = Registry()
    e = reg.create()
    reg.destroy(e)
    reg.emplace(e, owns Hp(n=1))
