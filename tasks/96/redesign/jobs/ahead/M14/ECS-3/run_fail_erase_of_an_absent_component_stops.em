# Clause ECS-3.2: "`erase[T](e)` (a stop when absent)"
#$ test: run-fail
#$ rules: ECS-3, STD-14
# The spec gives no stop text: the file claims the stop only.

from std.ecs import Registry

@derive(Copy)
struct Hp:
    n: int

fn main():
    reg = Registry()
    e = reg.create()
    reg.erase[Hp](e)
