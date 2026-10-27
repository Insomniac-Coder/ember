#$ test: compile-pass
#$ rules: R9, R15
## R9: one scoped task changes the `sharedref mut` target directly, since no other task and not the
## parent touch it before the join (the nearest allowed program to
## `reject_two_tasks_set_one_player_through_two_sharedref_links.em` and to
## `reject_an_unscoped_thread_holding_a_sharedref_link.em`).

import std.thread

struct Player:
    hp: int

struct Watcher:
    target: sharedref mut Player

fn main():
    p = Player(hp=10)
    a = Watcher(target=sharedref mut p)
    with scope = thread.scope():
        scope.spawn(fn(): a.target.hp = 1)
    println(p.hp)
