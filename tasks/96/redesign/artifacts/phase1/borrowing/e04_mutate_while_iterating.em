# A4: spawn while iterating the enemies.
struct Enemy:
    hp: int
    splits: bool

fn update(mut enemies: Array[Enemy]):
    for e in enemies:
        if e.hp <= 0 and e.splits:
            enemies.push(Enemy(hp=5, splits=false))

fn main():
    es = [Enemy(hp=0, splits=true)]
    update(es)
    println(es.len())
