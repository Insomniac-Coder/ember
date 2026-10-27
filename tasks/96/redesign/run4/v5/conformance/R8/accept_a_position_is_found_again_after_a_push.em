#$ test: run-pass
#$ rules: BRW-16, FN-1
#$ stdout: 2 50
## design rules: R8, R4
## R8 (a): a name for a list element made by indexing is a position, found again at each use; the list
## may grow while the name is used, and R8 (f) does not count the push as a use of the element.
## AN-01's NEW program.

struct Enemy:
    x: float
    hp: int

fn split_boss(ref mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))
    boss.hp = boss.hp // 2

fn main():
    es = [Enemy(x=0.0, hp=100)]
    split_boss(ref mut es)
    println(es.len(), es[0].hp)
