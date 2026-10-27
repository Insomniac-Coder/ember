#$ test: run-pass
#$ rules: R8, R5
#$ stdout:
#$ start
#$ 2
## R8 (c), R5, verdicts 57, 86: the way out the refusal gives: `reset`, which may end an `Enemy` whose
## `drop` pushes to `LOG`, is called after the loop over `LOG`. No first line names the change: the
## compiler works it out from `reset`'s body and `Enemy`'s `drop`. The nearest program to
## `reject_a_function_that_ends_an_enemy_called_inside_a_walk_of_log.em` that is allowed. Completeness
## check, G12.

static mut LOG: Array[String] = []

class Enemy:
    name: String

    fn drop():
        LOG.push(String("gone"))

fn reset(ref mut es: Array[Enemy]):
    es.clear()

fn main():
    enemies: Array[Enemy] = [Enemy(name=String("a"))]
    LOG.push(String("start"))
    for line in LOG:
        println(line)
    reset(ref mut enemies)
    println(LOG.len())
