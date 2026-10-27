#$ test: run-pass
#$ rules: R12, R4
#$ stdout: 2 1
## Verdicts 21 and 49, R12 (rules.md lines 659-665): the handler's type is named once with its change
## list as a parameter; `C` is `kills` for the lambda (main's own) and nothing for `drop_it`. AN-05
## point 3.

struct Enemy:
    hp: int

type DeathHandler[C] = fn(ref mut Array[Enemy], int) changes C

[C]
fn update(ref mut enemies: Array[Enemy], on_death: DeathHandler[C]) changes C:
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
    update(ref mut es, fn(ref mut list: Array[Enemy], i: int) changes kills: kills += 1)
    update(ref mut es, drop_it)
    println(kills, es.len())
