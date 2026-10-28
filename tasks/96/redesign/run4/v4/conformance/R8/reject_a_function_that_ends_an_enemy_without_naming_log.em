#$ test: compile-fail
#$ rules: R8, R5
#$ help: fn reset(ref mut es: Array[Enemy]) changes LOG:
## R5, verdict 57: a function that may end an `Enemy` makes the changes its `drop` names, and says them on
## its first line (E2105), so that a call of it inside a loop over `LOG` is seen (R8 (c)). Completeness
## check, G12.

static mut LOG: Array[String] = []

class Enemy:
    name: String

    fn drop() changes LOG:
        LOG.push(String("gone"))

fn reset(ref mut es: Array[Enemy]):    #$ error[E2105]: `reset` may end `Enemy` objects (`es.clear()`, line 17), and an `Enemy`'s `drop` pushes to `LOG`; add it to `reset`'s first line: `fn reset(ref mut es: Array[Enemy]) changes LOG:`
    es.clear()

fn main() changes LOG:
    enemies: Array[Enemy] = [Enemy(name=String("a"))]
    reset(ref mut enemies)
    println(LOG.len())
