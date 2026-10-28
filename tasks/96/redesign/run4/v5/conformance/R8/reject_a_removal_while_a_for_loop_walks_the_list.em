#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: enemies.retain(fn(e) => e.hp > 0)
## design rules: R8
## R8 (c): a removal from the list a `for` loop walks is refused; the dead are removed after the loop.
## AN-04, point 6.

struct Enemy:
    hp: int

fn update(ref mut enemies: Array[Enemy]):
    i = 0
    for e in enemies:
        if e.hp <= 0:
            enemies.remove(i)    #$ error[E3020]: `enemies.remove(i)` (line 14) shortens `enemies`, which this loop walks (line 12); remove the dead in one step instead: in place of lines 11 to 15 write `enemies.retain(fn(e) => e.hp > 0)`
        i += 1

fn main():
    es = [Enemy(hp=0), Enemy(hp=3)]
    update(ref mut es)
