#$ test: compile-fail
#$ rules: R8
#$ help: enemies.retain(fn(e) => e.hp > 0)
## R8 (c): a removal from the list a `for` loop walks is refused; the dead are removed after the loop.
## AN-04, point 6.

struct Enemy:
    hp: int

fn update(ref mut enemies: Array[Enemy]):
    i = 0
    for e in enemies:
        if e.hp <= 0:
            enemies.remove(i)    #$ error[E3020]: `enemies.remove(i)` shortens `enemies`, which this loop walks (line 12); remove the dead after the loop: `enemies.retain(fn(e) => e.hp > 0)`
        i += 1

fn main():
    es = [Enemy(hp=0), Enemy(hp=3)]
    update(ref mut es)
