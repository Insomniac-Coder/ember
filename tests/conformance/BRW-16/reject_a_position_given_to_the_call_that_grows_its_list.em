#$ test: compile-fail
#$ rules: BRW-16, BRW-11
# `[BRW-16]` — a position is found again after an add, but not inside the add: a call that adds to the list
# and is also given the position itself (or writes its result through it) would use the address the add may
# move, so the position is not made again after it: a later use is `E3060`.

struct Enemy:
    hp: int
    x: int

fn spawn_and_touch(ref mut es: Array[Enemy], ref mut b: Enemy):
    es.push(owns Enemy(hp=0, x=0))
    b.hp = 1

fn spawn_and_count(ref mut es: Array[Enemy]) -> int:
    es.push(owns Enemy(hp=0, x=0))
    return es.len()

fn given_the_position():
    enemies = [Enemy(hp=1, x=0)]
    boss = ref mut enemies[0]
    spawn_and_touch(ref mut enemies, ref mut boss)
    boss.hp = 2    #$ error[E3060]

fn written_through_the_position():
    enemies = [Enemy(hp=1, x=0)]
    boss = ref mut enemies[0]
    boss.x = spawn_and_count(ref mut enemies)
    boss.hp = 2    #$ error[E3060]

fn main():
    given_the_position()
    written_through_the_position()
