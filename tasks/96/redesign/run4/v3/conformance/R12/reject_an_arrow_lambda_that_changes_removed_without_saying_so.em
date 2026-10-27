#$ test: compile-fail
#$ rules: R12, R5
#$ help: enemies.retain(fn(e) changes removed => keep(e, ref mut removed))
## R12 (rules.md lines 519-521), verdict 45: a lambda written with `=>` says what it changes after its
## parameters, as one written with `:` does (`fn(e) changes removed => keep(e, ref mut removed)`); the
## message is the shape of rules.md section 3's E2105 row for a captured variable.

struct Enemy:
    hp: int

fn keep(e: Enemy, ref mut removed: int) -> bool:
    if e.hp > 0:
        return true
    removed += 1
    return false

fn update(ref mut enemies: Array[Enemy]) -> int:
    removed = 0
    enemies.retain(fn(e) => keep(e, ref mut removed))    #$ error[E2105]: this function changes `removed`, a variable it uses from `update`; say so on its first line: `enemies.retain(fn(e) changes removed => keep(e, ref mut removed))`
    return removed

fn main():
    es = [Enemy(hp=1), Enemy(hp=0), Enemy(hp=0)]
    n = update(ref mut es)
    println(n, es.len())
