#$ test: compile-fail
#$ rules: BRW-17
#$ help: boss = ref mut enemies[0]
## design rules: R8, R2
## R8 (a): `=` on the list's name replaces the list (R2), which ends every position of the old one
## (E3060 at a later use).

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = ref mut enemies[0]
    enemies = [Enemy(hp=20)]
    boss.hp -= 1    #$ error[E3060]: `enemies = [Enemy(hp=20)]` (line 13) replaces the list whose enemy `boss` names (line 12), and `boss` is used on line 14; use `boss` before line 13, or name the enemy again after it: `boss = ref mut enemies[0]`
