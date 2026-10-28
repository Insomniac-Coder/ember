#$ test: compile-fail
#$ rules: R8
#$ help: boss.hp = 1
## R8 (f), verdict 2: while a `ref mut` name is still used, its place is changed only through it.
## AN-01, point 6; the E3022 message of `V3/rules.md` section 3.

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = ref mut enemies[0]
    enemies[0].hp = 1    #$ error[E3022]: `boss` is the only name for `enemies[0]` until line 14, and line 13 changes `enemies[0].hp`; change it through `boss`: `boss.hp = 1`
    boss.hp -= 1
