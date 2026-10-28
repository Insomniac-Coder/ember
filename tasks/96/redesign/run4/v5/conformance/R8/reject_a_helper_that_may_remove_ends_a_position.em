#$ test: compile-fail
#$ rules: BRW-17
#$ help: boss = ref mut enemies[0]
## design rules: R8, R17
## R8 (a), (b): what may remove is read from the callee's body, its summary inside the package (R17);
## a helper that removes ends a position. AN-02, point 6. The helper reads no element already in the
## list, so the call is not also a read of `boss`'s place through another name (R8 (f)): one fault.

struct Enemy:
    hp: int

fn drop_last(ref mut enemies: Array[Enemy]):
    enemies.remove(enemies.len() - 1)

fn hit_boss(ref mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    boss.hp -= 10
    drop_last(ref mut enemies)
    boss.hp -= 10    #$ error[E3060]: `drop_last(ref mut enemies)` (line 17) may remove enemies (`drop_last` line 12: `enemies.remove(enemies.len() - 1)`), which may put another enemy in the place `boss` names (line 15), or none, and `boss` is used on line 18; use `boss` before line 17, or name the enemy again after it: `boss = ref mut enemies[0]`

fn main():
    es = [Enemy(hp=100), Enemy(hp=1)]
    hit_boss(ref mut es)
