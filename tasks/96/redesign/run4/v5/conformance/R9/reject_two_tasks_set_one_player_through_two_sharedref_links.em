#$ test: compile-fail
#$ rules: THR-11, THR-21
#$ help: lock: Lock = Lock()
## design rules: R9, R15
## R9: "A scoped task that reaches a `sharedref mut` target changes it directly only where neither
## another task nor the parent may touch it before the join, or by a reduction (R15); otherwise only
## as a `thread_shared` value is changed (E7007 ...)". `=` is not a reduction. A plain assignment from
## a task needs a lock (R15), which the message gives as whole lines (`V5/changes.md` section 8,
## item 17); the allowed neighbour with a reduction is
## `run_two_tasks_take_hp_off_one_player_through_two_sharedref_links.em`.

import std.thread

struct Player:
    hp: int

struct Watcher:
    target: sharedref mut Player

fn main():
    p = Player(hp=10)
    a = Watcher(target=sharedref mut p)
    b = Watcher(target=sharedref mut p)
    with scope = thread.scope():
        scope.spawn(fn(): a.target.hp = 1)
        scope.spawn(fn(): b.target.hp = 2)    #$ error[E7007]: two tasks set `p.hp` (lines 24 and 25) through `a.target` and `b.target`, links to one `Player`, with `=`, which is not a reduction, and without a lock; guard it: write `from std.sync import Lock` at the top, in `Player` `lock: Lock = Lock()` and `hp: int guarded by lock` in place of line 14, and `fn set_hp(n: int):` whose body is `with this.lock:` and, inside it, `this.hp = n`; then write `scope.spawn(fn(): a.target.set_hp(1))` in place of line 24, `scope.spawn(fn(): b.target.set_hp(2))` in place of line 25 and `with p.lock: println(p.hp)` in place of line 26
    println(p.hp)
