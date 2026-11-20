#$ test: compile-fail
#$ rules: CTL-1, FN-1
# `[CTL-1]` — `for ref mut x in e:` changes `e`'s elements, so `e` is a place this function may change.

struct Enemy:
    hp: int

fn heal(xs: Array[Enemy]):
    for ref mut e in xs:    #$ error[E3023]
        e.hp += 1

fn main():
    enemies = [Enemy(hp=1)]
    heal(enemies)
