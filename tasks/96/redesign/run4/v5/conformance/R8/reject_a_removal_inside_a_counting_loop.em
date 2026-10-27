#$ test: compile-fail
#$ rules: R8
#$ help: enemies.retain(fn(e) => e.hp > 0)
## R8 (c): a counting loop, whose range is fixed when it starts, may add to the list but not remove
## from it. The message of rules section 3, in this program's lines.

struct Enemy:
    hp: int

fn update(ref mut enemies: Array[Enemy]):
    for i in 0..enemies.len():
        if enemies[i].hp <= 0:
            enemies.remove(i)    #$ error[E3020]: `enemies.remove(i)` (line 13) shortens `enemies`, which `for i in 0..enemies.len():` (line 11) counts over with an end fixed when it started; remove after the loop: nothing else is left in the loop, so in place of lines 11 to 13 write `enemies.retain(fn(e) => e.hp > 0)`

fn main():
    es = [Enemy(hp=0), Enemy(hp=3)]
    update(ref mut es)
