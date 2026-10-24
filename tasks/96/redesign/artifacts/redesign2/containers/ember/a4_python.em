# A4 as a Python programmer writes it: spawn and despawn inside the update loop.
struct Enemy:
    hp: int
    splits: bool

fn update(mut enemies: Array[Enemy]):
    for e in enemies:
        e.hp -= 1
        if e.hp <= 0:
            if e.splits:
                enemies.push(Enemy(hp=5, splits=false))
            enemies.remove(e)

fn main():
    es = [Enemy(hp=1, splits=true), Enemy(hp=3, splits=false)]
    update(es)
    println(es.len())
