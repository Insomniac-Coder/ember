# A4, the copy loop without writes through e: does today's checker see the view across remove/push?
struct Enemy:
    hp: int

fn update(mut enemies: Array[Enemy]):
    for e in enemies[..]:
        if e.hp <= 0:
            _gone = enemies.remove(0)
            enemies.push(Enemy(hp=9))

fn main():
    es = [Enemy(hp=0), Enemy(hp=3)]
    update(es)
    println(es.len())
