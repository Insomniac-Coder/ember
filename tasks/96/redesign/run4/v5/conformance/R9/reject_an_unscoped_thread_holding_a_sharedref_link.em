#$ test: compile-fail
#$ rules: R9, R15, R7
## R9: "An unscoped thread never holds one (E7015: a lent value reaching another thread)": a
## `sharedref mut` link is a view of `main`'s player, and a thread from `thread.spawn` may outlive it.

import std.thread

struct Player:
    hp: int

struct Watcher:
    target: sharedref mut Player

fn main():
    p = Player(hp=10)
    a = Watcher(target=sharedref mut p)
    t = thread.spawn(fn() owns a: a.target.hp = 1)    #$ error[E7015]: the thread gets `a`, whose `target` is a link to `p`, a player of `main` on this thread, and a thread started with `thread.spawn` may outlive it; change it in a task that ends before `p` does: write `with scope = thread.scope():` and, inside it, `scope.spawn(fn(): a.target.hp = 1)` instead of lines 17 and 18
    t.join()
    println(p.hp)
