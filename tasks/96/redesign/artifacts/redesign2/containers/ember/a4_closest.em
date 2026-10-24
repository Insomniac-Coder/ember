# A4 in the closest form today's Ember has: iter_mut to write, remove by index.
struct Enemy:
    hp: int
    splits: bool

fn update(mut enemies: Array[Enemy]):
    for i, e in enumerate(enemies.iter_mut()):
        e.hp -= 1
        if e.hp <= 0:
            if e.splits:
                enemies.push(Enemy(hp=5, splits=false))
            _gone = enemies.remove(i)

fn main():
    es = [Enemy(hp=1, splits=true), Enemy(hp=3, splits=false)]
    update(es)
    println(es.len())
