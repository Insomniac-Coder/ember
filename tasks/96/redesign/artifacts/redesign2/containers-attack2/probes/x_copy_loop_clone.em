# A4, the Python habit with an explicit copy: for e in enemies.copy(). With struct elements the copy
# holds copies, so `e.hp -= 1` would change the copy, not the list.
struct Enemy:
    hp: int

fn update(mut enemies: Array[Enemy]):
    for e in enemies.clone():
        e.hp -= 1
        if e.hp <= 0:
            enemies.push(Enemy(hp=9))

fn main():
    es = [Enemy(hp=1), Enemy(hp=3)]
    update(es)
    println(es.len())
