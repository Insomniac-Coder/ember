# Clause BCK-6.1: "Because liveness is location-sensitive, these are accepted: a view whose last use
# precedes a later change (`[BRW-2]`); a list added to while a position of it is still used (`boss =
# ref mut enemies[0]`, `enemies.push(owns Enemy(hp=5))`, then `boss.hp = 1`); a view returned on one
# branch while the other branch changes the place it views, as in [`first_even`] and a loop that
# conditionally stores a view in a local declared outside it and changes the place it views on
# rounds where it did not."
#$ test: run-pass
#$ rules: BCK-6, BRW-2, BRW-16, BCK-2
#$ stdout: 3
#$ stdout: 1 2
#$ stdout: 4
#$ stdout: 0
#$ stdout: 12
#$ stdout: 0 0 12
# The four acceptances, each as the clause writes it; `first_even` is the spec's own function.

struct Enemy:
    hp: int

fn first_even(ref mut xs: MutSpan[int]) -> ref mut int:
    for i in 0..xs.len():
        if xs[i] % 2 == 0:
            return ref mut xs[i]
    xs[0] = 0
    return ref mut xs[0]

fn first_big(ref mut xs: Array[int]) -> int:
    found: Option[ref int] = None
    i = 0
    while i < xs.len():
        if xs[i] > 10:
            found = Some(ref xs[i])
            break
        xs[i] = 0
        i += 1
    if found exists as f:
        return f
    return -1

fn main():
    v = [1, 2]
    n = v.len()
    v.push(owns n)
    println(v.len())
    enemies = [Enemy(hp=3)]
    boss = ref mut enemies[0]
    enemies.push(owns Enemy(hp=5))
    boss.hp = 1
    println(enemies[0].hp, enemies.len())
    a = [3, 4, 5]
    println(first_even(ref mut a.as_mut_span()))
    b = [3, 5]
    println(first_even(ref mut b.as_mut_span()))
    c = [3, 5, 12]
    println(first_big(ref mut c))
    println(c[0], c[1], c[2])
