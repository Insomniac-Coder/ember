# Clause DIA-7.1: "Every ownership and view error is classified into a shape of §XVII.6.1 and emits
# that shape's help."
#$ test: compile-fail
#$ rules: DIA-7, DIA-3, BRW-1, BRW-5, BRW-15, BRW-17, BRW-18, BRW-21
# One refusal per shape, each with the spec's own message for it, whose fix is the shape's help:
# B1 (`if i != j:` above the call), B2 (the loop's own handle), B3 (`ref mut` on the name), B4
# (`ref mut` on the parameter and at the call), B7 (the use moved above, or a value of your own),
# B16 (`copy of` the argument). This file's line numbers in place of the examples'.

struct Enemy:
    hp: int

struct Body:
    x: int = 0

struct Clock:
    frame: int = 0

static mut LOG: Array[String] = []

fn collide(ref mut a: Body, ref mut b: Body):
    a.x += 1
    b.x -= 1

fn step(ref mut bodies: Array[Body], i: int, j: int):
    collide(ref mut bodies[i], ref mut bodies[j])    #$ error[E3022]: `collide` changes `a` and `b` as two different bodies, and `bodies[i]` and `bodies[j]` are the same body when `i == j`; write `if i != j:` above this line and indent it under it

fn tick(s: Clock):
    s.frame += 1    #$ error[E3023]: `tick` sets `s.frame`, but `s` only reads the caller's clock; write `fn tick(ref mut s: Clock):` and call `tick(ref mut clock)`

fn note(s: StrView):
    LOG.clear()
    println(s)

fn walk():
    xs = [1, 2, 3]
    for x in xs:
        println(x)
        xs[0] = 5    #$ error[E3020]: `xs[0] = 5` (line 39) writes an element of `xs`, which this loop walks (line 37); to change the element the loop is on, write `for ref mut x in xs:` in place of line 37 and `x = 5` in place of line 39

fn boss_fight():
    enemies = [Enemy(hp=3)]
    boss = enemies[0]
    println(boss.hp)
    enemies[0].hp = 1    #$ error[E3021]: `enemies[0].hp = 1` (line 45) changes the enemy `boss` reads (`boss = enemies[0]`, line 43), and `boss` is used on line 46; change it after `boss`'s last use, or write `boss = ref mut enemies[0]` and change it through `boss`: `boss.hp = 1`
    println(boss.hp)

fn keys_view():
    m: Map[String, int] = Map()
    m.insert(owns String("a"), owns 1)
    ks = sorted(m.keys())
    m.insert(owns String("z"), owns 1)
    println(ks[0])    #$ error[E3060]: `m.insert(owns String("z"), owns 1)` (line 52) changes `m`, whose keys `ks` (line 51) still views on line 53; use `ks` before line 52, or keep your own strings: write `ks = sorted(m.keys().copy_each())` in place of line 51

fn main():
    bodies = [Body(), Body()]
    step(ref mut bodies, 0, 1)
    clock = Clock()
    tick(clock)
    walk()
    boss_fight()
    keys_view()
    LOG.push(owns String("a"))
    note(LOG[0])    #$ error[E3031]: `note` clears `LOG` (its line 32) and then uses `s` (its line 33), and `LOG[0]` is part of `LOG`; give it a copy: `note(copy of LOG[0])`, or in `note` use `s` before `LOG.clear()`
