# Clause ECS-3.1: "`Registry` (EnTT's `entt::registry`) is a class: `create()` (...) `emplace(e, owns
# value: T) -> ref mut T`, `emplace_or_replace` (...), `replace` (...), `patch[T](e, f: fn(ref mut
# T)) -> ref mut T`, `insert(entities: Span[Entity], value: T)` for a `T: needs Clone`; `remove[T](e)
# -> int` (0 when absent) and `erase[T](e)` (...); `get[T](e) -> ref T` and `get_mut[T](e) -> ref mut
# T`, `try_get[T](e) -> Option[ref T]` and `try_get_mut[T](e) -> Option[ref mut T]`,
# `get_or_emplace(e, owns value: T) -> ref mut T`; `all_of[A, B](e)` and `any_of[A, B](e)`;
# `orphan(e)`; `storage[T]() -> ref mut Storage[T]`"; "Several types are listed in the brackets, any
# number of them (`[A, B]`, one alone `[A]`; variadic generics, `[TYP-47]`, verdict 115)"
#$ test: run-pass
#$ rules: ECS-3, TYP-47, LT-1
#$ stdout: 6
#$ stdout: 9
#$ stdout: 2 2
#$ stdout: 1 0
#$ stdout: true false
#$ stdout: true true
#$ stdout: false
#$ stdout: 4
#$ stdout: true
#$ stdout: 0
# Each method once, with one type and with two in its brackets.

from std.ecs import Registry, Entity

@derive(Copy, Clone)
struct Hp:
    n: int

@derive(Copy)
struct Armor:
    n: int

fn heal(ref mut h: Hp):
    h.n += 3

fn main():
    reg = Registry()
    a = reg.create()
    t = reg.create()
    reg.destroy(t)
    b = reg.create_at(t)
    h = reg.emplace(a, owns Hp(n=3))
    h.n += 3
    println(reg.get[Hp](a).n)
    reg.patch[Hp](a, heal)
    println(reg.get[Hp](a).n)
    reg.insert([b], Hp(n=2))
    reg.emplace_or_replace(a, owns Hp(n=2))
    println(reg.get[Hp](a).n, reg.get[Hp](b).n)
    println(reg.remove[Hp](b), reg.remove[Hp](b))
    println(reg.try_get[Hp](a).is_some(), reg.try_get[Hp](b).is_some())
    reg.emplace(a, owns Armor(n=1))
    println(reg.all_of[Hp, Armor](a), reg.any_of[Hp](a))
    println(reg.all_of[Armor](b))
    reg.get_or_emplace(b, owns Hp(n=4))
    if reg.try_get_mut[Hp](b) exists as m:
        println(m.n)
    reg.replace(b, owns Hp(n=5))
    reg.erase[Armor](a)
    reg.orphan(b)
    println(reg.valid(b))
    reg.clear_of[Hp]()
    reg.clear()
    n = 0
    for e, ref mut x in reg.storage[Hp]():
        n += 1
    println(n)
