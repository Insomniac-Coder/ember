#$ test: run-pass
#$ rules: BRW-16, CLO-1
#$ stdout: 2 2 5
## design rules: R8, R12
## R8 (c): the agreed way: the loop changes elements through `ref mut e`, and the removal and the
## addition go after it (`retain`, `extend`). AN-04's NEW program.

struct Enemy:
    hp: int
    splits: bool

fn update(ref mut enemies: Array[Enemy]):
    spawned: Array[Enemy] = []
    for ref mut e in enemies:
        e.hp -= 1
        if e.hp <= 0 and e.splits:
            spawned.push(Enemy(hp=5, splits=false))
    enemies.retain(fn(e) => e.hp > 0)
    enemies.extend(spawned)

fn main():
    es = [Enemy(hp=1, splits=true), Enemy(hp=3, splits=false)]
    update(ref mut es)
    println(es.len(), es[0].hp, es[1].hp)
