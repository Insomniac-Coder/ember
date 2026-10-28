#$ test: compile-fail
#$ rules: BRW-17
#$ help: boss = ref mut enemies[0]
## design rules: R17, R8
## R17: `ledger`'s interface file records that `kill_weak` does more than add to its `ref mut` list,
## so it may remove or reorder enemies, and `boss`, a position of that list, ends at the call (R8 (a);
## AN-02, E3060).

from ledger import Enemy, kill_weak

fn hit_boss(ref mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    boss.hp -= 10
    kill_weak(ref mut enemies)
    boss.hp -= 10    #$ error[E3060]: `kill_weak(ref mut enemies)` (line 13) may remove or reorder enemies (`ledger` records that `kill_weak` does more than add to its list), which may put another enemy in the place `boss` names (line 11), and `boss` is used on line 14; use `boss` before line 13, or name the enemy again after it: `boss = ref mut enemies[0]`

fn main():
    es = [Enemy(x=0.0, hp=100)]
    hit_boss(ref mut es)
    println(es.len())
