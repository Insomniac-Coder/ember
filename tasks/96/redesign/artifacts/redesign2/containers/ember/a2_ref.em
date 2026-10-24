# A2 with today's explicit reference: the same program with `ref mut`.
struct Enemy:
    x: float
    hp: int

fn split_boss(mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))
    boss.hp = boss.hp // 2

fn main():
    es = [Enemy(x=0.0, hp=100)]
    split_boss(es)
    println(es.len())
