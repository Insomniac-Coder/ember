#$ test: compile-fail
#$ rules: R8
#$ help: boss = ref mut enemies[0]
## R8 (g), verdict 5: a read-only name promises that what it reads does not change while it is used;
## the message gives both ways out. AN-01, point 6; the E3021 message of `V3/rules.md` section 3.

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = enemies[0]
    enemies[0].hp = 1    #$ error[E3021]: `enemies[0].hp = 1` (line 13) changes the enemy `boss` reads (`boss = enemies[0]`, line 12), and `boss` is used on line 14; change it after `boss`'s last use, or write `boss = ref mut enemies[0]` and change it through `boss`: `boss.hp = 1`
    println(boss.hp)
