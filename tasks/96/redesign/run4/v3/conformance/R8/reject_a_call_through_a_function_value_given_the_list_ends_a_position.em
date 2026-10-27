#$ test: compile-fail
#$ rules: R8, R12
#$ help: boss = ref mut enemies[0]
## R8 (a), (b): what may remove is read from the types of the function values a call may run; a
## `fn(ref mut Array[Enemy])` may change the list in any way, so it ends a position. AN-02, point 6.

struct Enemy:
    x: float
    hp: int

fn hit_boss(ref mut enemies: Array[Enemy], on_hit: fn(ref mut Array[Enemy])):
    boss = ref mut enemies[0]
    boss.hp -= 10
    on_hit(ref mut enemies)
    boss.hp -= 10    #$ error[E3060]: `on_hit(ref mut enemies)` (line 14) may remove or reorder enemies (its type, `fn(ref mut Array[Enemy])`, lets it change the list in any way), which may put another enemy in the place `boss` names (line 12), and `boss` is used on line 15; use `boss` before line 14, or name the enemy again after it: `boss = ref mut enemies[0]`

fn drop_last(ref mut list: Array[Enemy]):
    list.pop()

fn main():
    es = [Enemy(x=0.0, hp=100)]
    hit_boss(ref mut es, drop_last)
