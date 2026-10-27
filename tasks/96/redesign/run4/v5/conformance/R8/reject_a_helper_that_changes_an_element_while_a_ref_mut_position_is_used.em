#$ test: compile-fail
#$ rules: R8
#$ help: boss = ref mut enemies[0]
## R8 (f): while a `ref mut` name is used, its place is read or changed only through it; a helper that
## changes elements already in the list is another name for them. AN-02, point 6.

struct Enemy:
    hp: int

fn heal_all(ref mut enemies: Array[Enemy]):
    for ref mut e in enemies:
        e.hp += 1

fn hit_boss(ref mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    boss.hp -= 10
    heal_all(ref mut enemies)    #$ error[E3022]: `boss` is the only name for `enemies[0]` until line 18, and `heal_all(ref mut enemies)` (line 17) changes the enemies in it (`heal_all` line 12: `e.hp += 1`); use `boss` before line 17, or name the enemy again after it: `boss = ref mut enemies[0]`
    boss.hp -= 10

fn main():
    es = [Enemy(hp=100)]
    hit_boss(ref mut es)
