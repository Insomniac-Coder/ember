# Clause BRW-16.2: "the element is found again at each use through the list's current buffer. The
# list may grow while the name is used (its buffer is read again after the add)" and BRW-16.4:
# "Growing the list is not a use of the element."
#$ test: run-pass
#$ rules: BRW-16, BCK-3, BCK-6, LT-3, STD-15
#$ stdout: 1 101
#$ stdout: 7
#$ stdout: 3 2
#$ stdout: 4
#$ stdout: 5
# A hundred pushes move the buffer several times, and `boss` finds its enemy again after them; a
# read-only position the same. Then a list reached through a parameter, through `this` and a one-owner
# field, and a module variable.

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

static mut ALL: Array[Enemy] = [Enemy(hp=0)]

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
    mine = [Enemy(hp=0)]
    through_a_parameter(ref mut mine)
    level = Level(wave=Wave(enemies=[Enemy(hp=0)]))
    level.grow()
    a = ref mut ALL[0]
    ALL.push(owns Enemy(hp=1))
    a.hp = 5
    println(ALL[0].hp)
