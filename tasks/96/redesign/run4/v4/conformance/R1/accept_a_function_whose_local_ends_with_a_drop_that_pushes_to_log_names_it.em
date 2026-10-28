#$ test: run-pass
#$ rules: R1, R5
#$ stdout: 1
#$ 2
## R1, R5: the nearest program that is allowed: `spawn_one` names what the `drop` of the `Enemy` it
## ends changes (verdict 57); the `drop` runs once, when `spawn_one` returns.

static mut LOG: Array[String] = []

class Enemy:
    hp: int = 1

    fn drop() changes LOG:
        LOG.push(String("gone"))

fn spawn_one() changes LOG:
    e = Enemy()
    println(e.hp)

fn main() changes LOG:
    LOG.push(String("start"))
    spawn_one()
    println(LOG.len())
