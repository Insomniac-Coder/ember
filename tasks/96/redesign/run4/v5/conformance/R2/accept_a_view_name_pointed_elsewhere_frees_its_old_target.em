#$ test: run-pass
#$ rules: BRW-12, BRW-16
#$ stdout: bo cy
## design rules: R2, R8
## R2, R8 (g): the nearest program that is allowed: once `n` views `q.name`, `p.name` is free of it
## and may be replaced while `n` is still used.

struct Player:
    name: String

fn main():
    p = Player(name=String("ann"))
    q = Player(name=String("bo"))
    n = p.name
    n = q.name
    p.name = String("cy")
    println(n, p.name)
