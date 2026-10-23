# A2: keep a reference to an element while adding to the array.
struct Enemy:
    x: float
    hp: int

fn split_boss(mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp - 10))
    boss.hp -= 10

fn main():
    es = [Enemy(x=0.0, hp=100)]
    split_boss(es)
    println(es.len())
