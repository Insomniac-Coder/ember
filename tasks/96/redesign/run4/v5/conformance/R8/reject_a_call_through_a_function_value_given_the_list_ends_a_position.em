#$ test: compile-fail
#$ rules: BRW-17
#$ help: boss = ref mut enemies[0]
## design rules: R8, R12
## R8 (a), (b): what may remove is worked out from every function the program can put in a function
## value (R12, verdict 86): `on_hit` may run `drop_last`, which pops, so the call ends a position.
## AN-02, point 6.

struct Enemy:
    x: float
    hp: int

fn hit_boss(ref mut enemies: Array[Enemy], on_hit: fn(ref mut Array[Enemy])):
    boss = ref mut enemies[0]
    boss.hp -= 10
    on_hit(ref mut enemies)
    boss.hp -= 10    #$ error[E3060]: `on_hit(ref mut enemies)` (line 15) may remove enemies (it may run `drop_last`, given on line 23, which pops the list, its line 19), which may put another enemy in the place `boss` names (line 13), and `boss` is used on line 16; use `boss` before line 15, or name the enemy again after it: `boss = ref mut enemies[0]`

fn drop_last(ref mut list: Array[Enemy]):
    list.pop()

fn main():
    es = [Enemy(x=0.0, hp=100)]
    hit_boss(ref mut es, drop_last)
