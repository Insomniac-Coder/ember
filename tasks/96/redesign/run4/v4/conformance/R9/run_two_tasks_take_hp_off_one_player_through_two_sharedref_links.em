#$ test: run-pass
#$ rules: R9, R15, R7
#$ stdout: 7
## R9, R15 (verdict 77): two scoped tasks that change one place only by `-=` make a reduction: each task
## keeps its own part, and the parts are added at the join, so `p.hp` is always 10 - 1 - 2. It is the
## nearest allowed program to `reject_two_tasks_set_one_player_through_two_sharedref_links.em`, whose
## `=` is not a reduction (E7007). Checked under ThreadSanitizer: no report.

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
        scope.spawn(fn() changes Player: a.target.hp -= 1)
        scope.spawn(fn() changes Player: b.target.hp -= 2)
    println(p.hp)
