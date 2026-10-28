#$ test: compile-fail
#$ rules: R12, R4
#$ help: type DeathHandler[C] = fn(ref mut Array[Enemy], int) changes C
## Verdict 22, R12 (rules.md lines 608, 645-649): `fn(ref mut Array[Enemy], int)` changes nothing but the
## list it is given, so a lambda that changes a variable it captured does not fit it (E2113). AN-05
## part 6's message, word for word.

struct Enemy:
    hp: int

fn update(ref mut enemies: Array[Enemy], on_death: fn(ref mut Array[Enemy], int)):
    dead: Array[int] = []
    for i in 0..enemies.len():
        enemies[i].hp -= 1
        if enemies[i].hp <= 0:
            dead.push(i)
    while dead.len() > 0:
        on_death(ref mut enemies, dead.pop().unwrap())

fn main():
    es = [Enemy(hp=1), Enemy(hp=3)]
    kills = 0
    update(ref mut es, fn(ref mut list: Array[Enemy], i: int) changes kills: kills += 1)    #$ error[E2113]: this function changes `kills`, a variable it uses from `main` (`fn(ref mut list: Array[Enemy], i: int) changes kills`), and `update`'s `on_death` takes `fn(ref mut Array[Enemy], int)`, a function that changes nothing but the list it is given; to let `update` take handlers that change something, name its type with a change list, `type DeathHandler[C] = fn(ref mut Array[Enemy], int) changes C`, and write `update`'s first line as `[C]` above `fn update(ref mut enemies: Array[Enemy], on_death: DeathHandler[C]) changes C:`
    println(kills, es.len())
