#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: move line 26, `reset(ref mut enemies)`, below the loop
## design rules: R8, R5
## R8 (c), R5, verdicts 57, 86: ending an object counts as a call of its `drop`, and a function that may
## end one makes what that `drop` changes, worked out from its body with no word on its first line: so
## `reset(ref mut enemies)`, which clears the enemies, pushes to `LOG` through `Enemy`'s `drop`, and
## inside a loop over `LOG` it is refused at the call, the message naming the chain. It replaces the E2105
## test `reject_a_function_that_ends_an_enemy_without_naming_log.em`, whose word is gone (verdict 86).
## Completeness check, G12.

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
        reset(ref mut enemies)    #$ error[E3020]: `reset(ref mut enemies)` (line 26) ends every `Enemy` in `enemies` (through `reset`, `es.clear()`, line 20), and an `Enemy`'s `drop` pushes to `LOG` (line 17), which this loop walks (line 25); call it after the loop: move line 26, `reset(ref mut enemies)`, below the loop
        println(line)
