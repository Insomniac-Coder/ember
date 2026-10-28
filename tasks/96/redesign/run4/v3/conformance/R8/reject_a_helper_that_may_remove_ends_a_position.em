#$ test: compile-fail
#$ rules: R8, R17
#$ help: boss = ref mut enemies[0]
## R8 (a), (b): what may remove is read from the callee's body, its summary inside the package (R17);
## a helper that removes ends a position. AN-02, point 6.

struct Enemy:
    hp: int

fn kill_weak(ref mut enemies: Array[Enemy]):
    i = 0
    while i < enemies.len():
        if enemies[i].hp < 5:
            enemies.remove(i)
        else:
            i += 1

fn hit_boss(ref mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    boss.hp -= 10
    kill_weak(ref mut enemies)
    boss.hp -= 10    #$ error[E3060]: `kill_weak(ref mut enemies)` (line 21) may remove enemies (`kill_weak` line 14: `enemies.remove(i)`), which may put another enemy in the place `boss` names (line 19), and `boss` is used on line 22; use `boss` before line 21, or name the enemy again after it: `boss = ref mut enemies[0]`

fn main():
    es = [Enemy(hp=100), Enemy(hp=1)]
    hit_boss(ref mut es)
