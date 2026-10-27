#$ test: run-pass
#$ rules: BRW-18, CTL-2
#$ stdout:
#$ start
#$ 2
## design rules: R8, R5
## R8 (c), verdict 57: the way out E3020's message gives: the enemies are cleared after the loop over
## `LOG`, and each `Enemy`'s `drop` then pushes to it. The nearest program to
## `reject_ending_an_object_whose_drop_pushes_to_the_walked_list.em` that is allowed. Completeness
## check, G12.

static mut LOG: Array[String] = []

class Enemy:
    name: String

    fn drop():
        LOG.push(String("gone"))

fn main():
    enemies: Array[Enemy] = [Enemy(name=String("a"))]
    LOG.push(String("start"))
    for line in LOG:
        println(line)
    enemies.clear()
    println(LOG.len())
