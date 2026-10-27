#$ test: run-pass
#$ rules: CLO-1, FN-1
#$ stdout: 2 1
## design rules: R12, R4
## Verdicts 49 and 86, R12 (rules.md lines 672-674: "`type DeathHandler = fn(ref mut Array[Enemy],
## int)`"): the long function type is named once with `type`, with no change list; `update` runs the
## handler it is given, and each call makes what that handler changes, worked out from it: `main`'s own
## `kills` for the lambda, the list it is given for `drop_it`. AN-05 point 3.

struct Enemy:
    hp: int

type DeathHandler = fn(ref mut Array[Enemy], int)

fn update(ref mut enemies: Array[Enemy], on_death: DeathHandler):
    dead: Array[int] = []
    for i in 0..enemies.len():
        enemies[i].hp -= 1
        if enemies[i].hp <= 0:
            dead.push(i)
    while dead.len() > 0:
        on_death(ref mut enemies, dead.pop().unwrap())

fn drop_it(ref mut list: Array[Enemy], i: int):
    _gone = list.remove(i)

fn main():
    es = [Enemy(hp=1), Enemy(hp=3), Enemy(hp=1)]
    kills = 0
    update(ref mut es, fn(ref mut list: Array[Enemy], i: int): kills += 1)
    update(ref mut es, drop_it)
    println(kills, es.len())
