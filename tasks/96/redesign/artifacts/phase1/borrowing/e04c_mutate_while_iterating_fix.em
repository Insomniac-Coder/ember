# A4 workaround: collect spawns, then retain and extend.
struct Enemy:
    hp: int
    splits: bool

fn update(mut enemies: Array[Enemy]):
    spawned: Array[Enemy] = []
    for e in enemies:
        if e.hp <= 0 and e.splits:
            spawned.push(Enemy(hp=5, splits=false))
    enemies.retain(fn(e: Enemy) => e.hp > 0)
    for s in owned spawned:
        enemies.push(s)

fn main():
    es = [Enemy(hp=0, splits=true)]
    update(es)
    println(es.len())
