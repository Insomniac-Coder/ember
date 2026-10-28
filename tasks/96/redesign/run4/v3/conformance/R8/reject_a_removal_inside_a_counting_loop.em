#$ test: compile-fail
#$ rules: R8
#$ help: enemies.retain(fn(e) => e.hp > 0)
## R8 (c): a counting loop, whose range is fixed when it starts, may add to the list but not remove
## from it. The message of `V3/rules.md` section 3.

struct Enemy:
    hp: int

fn update(ref mut enemies: Array[Enemy]):
    for i in 0..enemies.len():
        if enemies[i].hp <= 0:
            enemies.remove(i)    #$ error[E3020]: `enemies.remove(i)` shortens `enemies`, which `for i in 0..enemies.len():` counts over with an end fixed when it started; remove after the loop: `enemies.retain(fn(e) => e.hp > 0)`

fn main():
    es = [Enemy(hp=0), Enemy(hp=3)]
    update(ref mut es)
