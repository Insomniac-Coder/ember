#$ test: compile-fail
#$ rules: BRW-17
#$ help: move `enemies.clear()` below line 13
## design rules: R8
## R8 (a): `clear` empties the list, so it ends every position of it (E3060 at a later use).

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = ref mut enemies[0]
    enemies.clear()
    boss.hp -= 1    #$ error[E3060]: `enemies.clear()` (line 12) empties `enemies`, so the place `boss` names (line 11) is gone, and `boss` is used on line 13; move `enemies.clear()` below line 13, after `boss`'s last use
