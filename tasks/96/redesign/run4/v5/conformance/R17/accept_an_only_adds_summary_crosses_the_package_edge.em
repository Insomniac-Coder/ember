#$ test: run-pass
#$ rules: BLD-14, BRW-16
#$ stdout: 2 80
## design rules: R17, R8
## R17: where no word exists for a fact, it is recorded in the package's interface file: `ledger`'s
## `spawn_minion` only adds at the end of its `ref mut` list, so `boss`, a position of that list, stays
## usable across the call (R8 (a), (f)). The fact decides only that the line compiles: `boss` names
## element 0 either way (AN-02, across a package's edge).

from ledger import Enemy, spawn_minion

fn hit_boss(ref mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    boss.hp -= 10
    spawn_minion(ref mut enemies, boss.x)
    boss.hp -= 10

fn main():
    es = [Enemy(x=0.0, hp=100)]
    hit_boss(ref mut es)
    println(es.len(), es[0].hp)
