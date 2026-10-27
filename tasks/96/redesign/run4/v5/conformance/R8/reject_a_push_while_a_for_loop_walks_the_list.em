#$ test: compile-fail
#$ rules: R8
#$ help: spawned: Array[Enemy] = []
## R8 (c): a list may not change while a `for` loop walks it; the change goes after the loop. AN-04,
## point 6; the message of rules section 3.

struct Enemy:
    hp: int
    splits: bool

fn update(ref mut enemies: Array[Enemy]):
    for ref mut e in enemies:
        e.hp -= 1
        if e.hp <= 0 and e.splits:
            enemies.push(Enemy(hp=5, splits=false))    #$ error[E3020]: `enemies.push(...)` adds to `enemies`, which this loop walks (line 12); collect the new enemies in a list of your own: write `spawned: Array[Enemy] = []` before the loop, `spawned.push(Enemy(hp=5, splits=false))` in place of this line, and `enemies.extend(spawned)` after the loop

fn main():
    es = [Enemy(hp=1, splits=true)]
    update(ref mut es)
