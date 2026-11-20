#$ test: run-pass
#$ rules: CTL-1, CTL-2
#$ stdout: [5, 5, 5]
#$ stdout: 11 12
#$ stdout: [2, 4, 6]
#$ stdout: 7 7
#$ stdout: a 2
#$ stdout: b 4
#$ stdout: a 2
# `[CTL-1]` — `for ref mut x in e:` names the element so that it may change it (`for ref mut x in xs: x = 5`
# gives `[5, 5, 5]`), over a list, a fixed array or a `MutSpan`; a `Map` with two names gives its keys
# and values, and `for k, ref mut v in counts: v *= 2` changes each value (`[CTL-2]`).

struct Enemy:
    hp: int

fn main():
    xs = [1, 2, 3]
    for ref mut x in xs:
        x = 5
    println(xs)
    enemies = [Enemy(hp=1), Enemy(hp=2)]
    for ref mut e in enemies:
        e.hp += 10
    println(enemies[0].hp, enemies[1].hp)
    ns = [1, 2, 3]
    view = ns.as_mut_span()
    for ref mut n in view:
        n *= 2
    println(ns)
    fixed: [int; 2] = [3, 4]
    for ref mut f in fixed:
        f = 7
    println(fixed[0], fixed[1])
    counts = Map[String, int]()
    counts.insert(owns String("a"), owns 1)
    counts.insert(owns String("b"), owns 2)
    for _k, ref mut v in counts:
        v *= 2
    for k, v in counts:
        println(k, v)
    for k in counts:
        if k == "a":
            println(k, counts[k])
