#$ test: run-pass
#$ rules: BRW-16, STD-15
#$ stdout: 1 101
#$ stdout: 7
#$ stdout: 77 50
#$ stdout: 3 2
#$ stdout: 4
# `[BRW-16]` — a name for a list element made by indexing is a position: the index is read once, where the
# name is made (a later change of `i` moves nothing), and the element is found again through the list's
# current buffer after an add. A hundred pushes move the buffer several times; a read-only position the
# same; a list reached through a parameter, and through `this` and a one-owner field.

struct Enemy:
    hp: int

struct Wave:
    enemies: Array[Enemy]

class Level:
    wave: Wave
    fn grow():
        first = ref mut this.wave.enemies[0]
        this.wave.enemies.push(owns Enemy(hp=0))
        first.hp = 4
        println(this.wave.enemies[0].hp)

fn through_a_parameter(ref mut es: Array[Enemy]):
    e = ref mut es[0]
    es.push(owns Enemy(hp=2))
    e.hp = 3
    println(es[0].hp, es[1].hp)

fn main():
    enemies = [Enemy(hp=9)]
    boss = ref mut enemies[0]
    for k in 0..100:
        enemies.push(owns Enemy(hp=k))
    boss.hp = 1
    println(enemies[0].hp, enemies.len())
    b = enemies[0]
    enemies.push(owns Enemy(hp=0))
    println(b.hp + 6)
    i = 1
    second = ref mut enemies[i]
    i = 50
    enemies.push(owns Enemy(hp=0))
    second.hp = 77
    println(enemies[1].hp, i)
    mine = [Enemy(hp=0)]
    through_a_parameter(ref mut mine)
    level = Level(wave=Wave(enemies=[Enemy(hp=0)]))
    level.grow()
