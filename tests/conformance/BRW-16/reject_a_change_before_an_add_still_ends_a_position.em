#$ test: compile-fail
#$ rules: BRW-16
# `[BRW-16]` — a position is made again after an add, but what ended it before the add still ends it: a
# clear, then a push, then a use is `E3060` at the use (the push makes no position the clear ended).

struct Enemy:
    hp: int

fn cleared_then_grown():
    xs = [Enemy(hp=1), Enemy(hp=2), Enemy(hp=3)]
    boss = ref mut xs[2]
    xs.clear()
    xs.push(owns Enemy(hp=9))
    boss.hp = 7    #$ error[E3060]: `xs.clear()` (line 12) may end the enemy `boss` names (line 11), and `boss` is used on line 14; use `boss` before line 12, or name the enemy again after it: `boss = ref mut xs[2]`
    println(xs.len())

fn removed_in_a_loop_that_grows():
    xs = [Enemy(hp=1), Enemy(hp=2)]
    boss = xs[1]
    for k in 0..3:
        _gone = xs.pop()
        xs.push(owns Enemy(hp=k))
    println(boss.hp)    #$ error[E3060]

fn main():
    cleared_then_grown()
    removed_in_a_loop_that_grows()
