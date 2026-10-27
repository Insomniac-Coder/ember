#$ test: compile-fail
#$ rules: BRW-1
#$ help: boss.hp
## design rules: R8
## R8 (f), verdict 2: while a `ref mut` name is still used, its place is not read through another name
## either. The E3021 message of rules section 3.

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = ref mut enemies[0]
    x = enemies[0].hp    #$ error[E3021]: `boss` is the only name for `enemies[0]` until line 14, and line 13 reads `enemies[0].hp`; read it through `boss`: `boss.hp`
    boss.hp -= x
