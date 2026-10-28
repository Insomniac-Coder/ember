#$ test: run-pass
#$ rules: R8, R5
#$ stdout: 1
## R5, verdict 57: `reset` may end an `Enemy`, whose `drop` pushes to `LOG`, and says `changes LOG`. The
## nearest program to `reject_a_function_that_ends_an_enemy_without_naming_log.em` that is allowed.
## Completeness check, G12.

static mut LOG: Array[String] = []

class Enemy:
    name: String

    fn drop() changes LOG:
        LOG.push(String("gone"))

fn reset(ref mut es: Array[Enemy]) changes LOG:
    es.clear()

fn main() changes LOG:
    enemies: Array[Enemy] = [Enemy(name=String("a"))]
    reset(ref mut enemies)
    println(LOG.len())
