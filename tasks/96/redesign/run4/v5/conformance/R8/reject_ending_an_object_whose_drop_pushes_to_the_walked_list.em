#$ test: compile-fail
#$ rules: R8, R5
#$ help: move line 20, `enemies.clear()`, below the loop
## R8 (c), R5, verdict 57: ending an object counts as a call of its `drop`; `enemies.clear()` ends an
## `Enemy`, whose `drop` pushes to `LOG`, so inside a loop over `LOG` it is refused at the change
## (E3020). Completeness check, G12.

static mut LOG: Array[String] = []

class Enemy:
    name: String

    fn drop():
        LOG.push(String("gone"))

fn main():
    enemies: Array[Enemy] = [Enemy(name=String("a"))]
    LOG.push(String("start"))
    for line in LOG:
        enemies.clear()    #$ error[E3020]: `enemies.clear()` (line 20) ends every `Enemy` in `enemies`, and an `Enemy`'s `drop` pushes to `LOG` (its line 14), which this loop walks (line 19); clear after the loop: move line 20, `enemies.clear()`, below the loop
        println(line)
