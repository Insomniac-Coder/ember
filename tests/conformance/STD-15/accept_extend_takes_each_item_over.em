#$ test: run-pass
#$ rules: STD-15
#$ stdout: 3 3
#$ stdout: [1, 7, 8, 10, 11]
#$ stdout: ['a', 'b'] ['b']
# `[STD-15]` — `extend(owns items)` takes each item of an iterable of `T` over, as `push(owns value: T)`
# takes its value: a list of unique owners, a list made on the line, a range; a caller that keeps its
# list writes `xs.extend(owns copy of ys)`.

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=1)]
    spawned = [Enemy(hp=2), Enemy(hp=3)]
    enemies.extend(owns spawned)
    println(enemies.len(), enemies[2].hp)
    ns = [1]
    ns.extend(owns [7, 8])
    ns.extend(owns (10..12))
    println(ns)
    names = [String("a")]
    more = [String("b")]
    names.extend(owns copy of more)
    println(names, more)
