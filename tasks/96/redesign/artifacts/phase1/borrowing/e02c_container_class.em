# A2 with class elements: the element is a counted handle, so pushing cannot leave it dangling.
class Enemy:
    x: float
    hp: int

fn split_boss(mut enemies: Array[Enemy]):
    boss = enemies[0]
    enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp - 10))
    boss.hp -= 10

fn main():
    es = [Enemy(x=0.0, hp=100)]
    split_boss(es)
    println(es.len(), es[0].hp)
