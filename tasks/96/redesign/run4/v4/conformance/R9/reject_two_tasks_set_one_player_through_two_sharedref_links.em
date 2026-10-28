#$ test: compile-fail
#$ rules: R9, R15
## R9: "A scoped task that reaches a `sharedref mut` target changes it directly only where neither
## another task nor the parent may touch it before the join; otherwise only as a `thread_shared` value
## is changed (E7007, two tasks or a task and the parent changing one place)". `=` is not a reduction.

import std.thread

struct Player:
    hp: int

struct Watcher:
    target: sharedref mut Player

fn main() changes Player:
    p = Player(hp=10)
    a = Watcher(target=sharedref mut p)
    b = Watcher(target=sharedref mut p)
    with scope = thread.scope():
        scope.spawn(fn() changes Player: a.target.hp = 1)
        scope.spawn(fn() changes Player: b.target.hp = 2)    #$ error[E7007]: two tasks change `p.hp` (lines 20 and 21), through `a.target` and `b.target`, links to one `Player`, each without a lock; make it an atomic: write `from std.sync import Atomic` at the top, `hp: Atomic[int]` in `Player` instead of line 10, `fn main() changes Atomic[int]:` instead of line 15, `p = Player(hp=Atomic(10))` instead of line 16, `scope.spawn(fn() changes Atomic[int]: a.target.hp.store(1))` instead of line 20, `scope.spawn(fn() changes Atomic[int]: b.target.hp.store(2))` instead of line 21 and `println(p.hp.load())` instead of line 22
    println(p.hp)
