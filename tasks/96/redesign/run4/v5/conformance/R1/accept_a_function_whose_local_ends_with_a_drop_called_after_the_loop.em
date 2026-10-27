#$ test: run-pass
#$ rules: OWN-1, FN-4, BRW-16
#$ stdout: start
#$ 1
#$ 2
## design rules: R1, R5, R8
## R1, R5, R8 (c): the nearest program that is allowed: `spawn_one()` runs after the loop over `LOG`, so
## the push its local's `drop` makes (verdict 57) changes no list a loop walks; no first line names it
## (verdict 86). The `drop` runs once, when `spawn_one` returns.

static mut LOG: Array[String] = []

class Enemy:
    hp: int = 1

    fn drop():
        LOG.push(String("gone"))

fn spawn_one():
    e = Enemy()
    println(e.hp)

fn main():
    LOG.push(String("start"))
    for line in LOG:
        println(line)
    spawn_one()
    println(LOG.len())
