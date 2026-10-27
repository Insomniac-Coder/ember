#$ test: compile-fail
#$ rules: R8
#$ help: boss = ref mut enemies[0]
## R8 (a): anything that may remove, insert, reorder, clear or replace the list ends a position (E3060
## at a later use). AN-01, point 6. A removal through another name while `boss`, a `ref mut` name, is
## still used is E3060, not E3022 (R8 (f), R18: the rule that names the case decides, verdict 73).

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10), Enemy(hp=20)]
    boss = ref mut enemies[0]
    enemies.remove(0)
    boss.hp -= 1    #$ error[E3060]: `enemies.remove(0)` (line 14) may put another enemy in the place `boss` names (line 13), and `boss` is used on line 15; use `boss` before line 14, or name the enemy again after it: `boss = ref mut enemies[0]`
