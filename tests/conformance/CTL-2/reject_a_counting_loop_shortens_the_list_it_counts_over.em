#$ test: compile-fail
#$ rules: CTL-2
# `[CTL-2]` — a `for` over a range whose header reads `xs.len()` counts over `xs`: it may write its elements
# and add at the end, but a call in its body that shortens `xs` is `E3020` ("`xs.pop()` shortens `xs`, whose
# length this loop read once on line 3; walk backwards, or collect the indices and remove after the loop").

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=0), Enemy(hp=5)]
    for i in 0..enemies.len():
        enemies[i].hp += 1
        enemies.push(owns Enemy(hp=9))
    for i in 0..enemies.len():
        if enemies[i].hp <= 1:
            _gone = enemies.remove(i)    #$ error[E3020]: `enemies.remove(i)` shortens `enemies`, whose length this loop read once on line 15; walk backwards, or collect the indices and remove after the loop
    xs = [1, 2, 3]
    for _i in 0..xs.len():
        _last = xs.pop()    #$ error[E3020]: `xs.pop()` shortens `xs`, whose length this loop read once on line 19
