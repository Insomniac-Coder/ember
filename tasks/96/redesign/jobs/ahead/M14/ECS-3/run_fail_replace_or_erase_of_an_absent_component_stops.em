# Clause ECS-3.2: "`replace(e, owns value: T) -> ref mut T` (a stop when `e` has no `T`)"; "`erase[T](e)`
# (a stop when absent)"
#$ test: run-fail
#$ rules: ECS-3, STD-14
# The spec gives no stop text: the file claims the stop only. The `erase` case is
# `run_fail_erase_of_an_absent_component_stops`.

from std.ecs import Registry

@derive(Copy)
struct Hp:
    n: int

fn main():
    reg = Registry()
    e = reg.create()
    reg.replace(e, owns Hp(n=1))
