#$ test: run-pass
#$ rules: R8, R17
#$ stdout: 2 80
## R8 (a): a callee whose summary is "only adds to `enemies`" grows the list that holds `boss`'s
## position, which `boss` survives, and it is not a use of element 0 through another name (R8 (f)).
## AN-02's NEW program.

struct Enemy:
    x: float
    hp: int

fn spawn_minion(ref mut enemies: Array[Enemy], x: float):
    enemies.push(Enemy(x=x, hp=10))

fn hit_boss(ref mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    boss.hp -= 10
    spawn_minion(ref mut enemies, boss.x)
    boss.hp -= 10

fn main():
    es = [Enemy(x=0.0, hp=100)]
    hit_boss(ref mut es)
    println(es.len(), es[0].hp)
