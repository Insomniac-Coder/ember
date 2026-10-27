#$ test: compile-fail
#$ rules: R8
#$ help: boss = ref mut enemies[0]
## R8 (a): an insertion may put another element in a position's place, so it ends the position (E3060
## at a later use).

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = ref mut enemies[0]
    enemies.insert(0, Enemy(hp=5))
    boss.hp -= 1    #$ error[E3060]: `enemies.insert(0, Enemy(hp=5))` (line 13) may put another enemy in the place `boss` names (line 12), and `boss` is used on line 14; use `boss` before line 13, or name the enemy again after it: `boss = ref mut enemies[0]`
