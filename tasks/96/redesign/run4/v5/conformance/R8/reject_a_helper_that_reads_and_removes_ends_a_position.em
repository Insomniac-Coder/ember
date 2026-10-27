#$ test: compile-fail
#$ rules: BRW-17
#$ help: boss = ref mut enemies[0]
## design rules: R8
## R8 (a), (f): a helper that reads elements of the list and removes from it, called while `boss`, a
## `ref mut` position of that list, is still used, ends the position: E3060 at the later use, by R8 (a),
## the rule that names a position after a removal (R18), not E3021 at the call. Settled in step 4
## (`V5/changes.md` section 8, item 11). The allowed neighbour is
## `accept_a_position_named_again_after_a_removal.em`.

struct Enemy:
    hp: int

fn kill_weak(ref mut enemies: Array[Enemy]):
    if enemies[enemies.len() - 1].hp < 5:
        enemies.pop()

fn hit_boss(ref mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    boss.hp -= 10
    kill_weak(ref mut enemies)
    boss.hp -= 10    #$ error[E3060]: `kill_weak(ref mut enemies)` (line 20) may remove enemies (`kill_weak` line 15: `enemies.pop()`), which may put another enemy in the place `boss` names (line 18), or none, and `boss` is used on line 21; use `boss` before line 20, or name the enemy again after it: `boss = ref mut enemies[0]`

fn main():
    es = [Enemy(hp=100), Enemy(hp=1)]
    hit_boss(ref mut es)
